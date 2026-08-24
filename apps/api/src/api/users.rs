use crate::api::AppState;
use crate::api::error::AppError;
use crate::api::middleware::{
    Claims, TokenPurpose, UserContext, create_token, create_token_with_jti,
};
use crate::domain::user::{User, UserResponse};
use crate::infrastructure::db::refresh_session_repo::RefreshSessionRepository;
use crate::infrastructure::db::user_repo::UserRepository;
use crate::utils::crypto::{hash_password, verify_password};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, Validation, decode};
use mongodb::bson::{Bson, DateTime as BsonDateTime, doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const ACCESS_TTL_SECS: i64 = 3600 * 2;
const REFRESH_TTL_SECS: i64 = 3600 * 24 * 7;

fn invalid_token() -> AppError {
    AppError::Unauthorized("Invalid token".to_string())
}

struct IssuedTokens {
    access_token: String,
    refresh_token: String,
    refresh_jti: String,
    refresh_expires_at: mongodb::bson::DateTime,
}

fn issue_auth_tokens(
    secret: &str,
    user_id: &str,
    token_version: i32,
) -> Result<IssuedTokens, AppError> {
    let now = Utc::now().timestamp();
    let refresh_exp = now + REFRESH_TTL_SECS;
    let refresh_jti = uuid::Uuid::new_v4().to_string();
    let access_token = create_token(
        secret,
        user_id,
        TokenPurpose::Access,
        None,
        (now + ACCESS_TTL_SECS) as usize,
        None,
        Some(token_version),
    )?;
    let refresh_token = create_token_with_jti(
        secret,
        user_id,
        TokenPurpose::Refresh,
        None,
        refresh_exp as usize,
        None,
        Some(token_version),
        Some(refresh_jti.clone()),
    )?;
    Ok(IssuedTokens {
        access_token,
        refresh_token,
        refresh_jti,
        refresh_expires_at: mongodb::bson::DateTime::from_millis(refresh_exp * 1000),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
    #[serde(rename = "firstName")]
    #[schema(example = "John")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    #[schema(example = "Doe")]
    pub last_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserDto {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub gender: Option<i32>,
    pub age: Option<i32>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatePasswordDto {
    pub password: String,
    #[serde(rename = "changePassword")]
    pub change_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub id: String,
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid request body (e.g. missing required field 'email' or 'password')"),
        (status = 401, description = "Invalid email or password")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = UserRepository::new(&state.db);
    let user = match repo.find_by_email(&payload.email).await? {
        Some(u) => u,
        None => {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }
    };

    if !verify_password(&payload.password, &user.password) {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let user_id = user
        .id
        .ok_or_else(|| AppError::DatabaseError(mongodb::error::Error::custom("missing id")))?
        .to_hex();
    let tokens = issue_auth_tokens(&state.config.jwt_secret, &user_id, user.token_version)?;
    RefreshSessionRepository::new(&state.db)
        .create(
            &user_id,
            &tokens.refresh_jti,
            user.token_version,
            tokens.refresh_expires_at,
        )
        .await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/auth/refresh",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Tokens refreshed", body = LoginResponse),
        (status = 401, description = "Invalid refresh token")
    ),
    tag = "auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenDto>,
) -> Result<impl IntoResponse, AppError> {
    let claims = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| invalid_token())?
    .claims;

    if claims.purpose != TokenPurpose::Refresh {
        return Err(invalid_token());
    }

    let token_version = claims.token_version.ok_or_else(invalid_token)?;
    let refresh_jti = claims.jti.as_deref().ok_or_else(invalid_token)?;
    let id = ObjectId::parse_str(&claims.user_id).map_err(|_| invalid_token())?;
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(id).await?.ok_or_else(invalid_token)?;
    if user.token_version != token_version {
        return Err(invalid_token());
    }

    let sessions = RefreshSessionRepository::new(&state.db);
    if !sessions
        .consume(&claims.user_id, refresh_jti, token_version)
        .await?
    {
        return Err(invalid_token());
    }

    let user_id = user.id.ok_or_else(invalid_token)?.to_hex();
    let tokens = issue_auth_tokens(&state.config.jwt_secret, &user_id, user.token_version)?;
    sessions
        .create(
            &user_id,
            &tokens.refresh_jti,
            user.token_version,
            tokens.refresh_expires_at,
        )
        .await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = CreateUserResponse),
        (status = 400, description = "User already exists, or invalid body (required: email, password, firstName, lastName)")
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = UserRepository::new(&state.db);

    // Check if user exists
    if repo.find_by_email(&payload.email).await?.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    let user = User {
        id: None,
        email: payload.email,
        password: hash_password(&payload.password),
        first_name: payload.first_name,
        last_name: payload.last_name,
        gender: None,
        age: None,
        avatar: None,
        roles: vec!["user".to_string()],
        token_version: 0,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };

    let id = repo.create(user).await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse { id: id.to_hex() }),
    ))
}

#[utoipa::path(
    patch,
    path = "/v1/profile",
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<UpdateUserDto>,
) -> impl IntoResponse {
    let id = match ObjectId::parse_str(&user_ctx.user_id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID").into_response(),
    };

    let mut update = doc! { "updatedAt": Bson::DateTime(BsonDateTime::from_chrono(Utc::now())) };
    let mut has_update = false;

    if let Some(f) = payload.first_name {
        let f = f.trim();
        if f.is_empty() || f.len() > 100 {
            return (StatusCode::BAD_REQUEST, "Invalid firstName").into_response();
        }
        update.insert("firstName", f);
        has_update = true;
    }
    if let Some(l) = payload.last_name {
        let l = l.trim();
        if l.is_empty() || l.len() > 100 {
            return (StatusCode::BAD_REQUEST, "Invalid lastName").into_response();
        }
        update.insert("lastName", l);
        has_update = true;
    }
    if let Some(g) = payload.gender {
        update.insert("gender", g);
        has_update = true;
    }
    if let Some(a) = payload.age {
        if !(0..=150).contains(&a) {
            return (StatusCode::BAD_REQUEST, "Invalid age").into_response();
        }
        update.insert("age", a);
        has_update = true;
    }
    if let Some(av) = payload.avatar {
        if av.len() > 2048 {
            return (StatusCode::BAD_REQUEST, "Invalid avatar").into_response();
        }
        update.insert("avatar", av);
        has_update = true;
    }

    if !has_update {
        return (StatusCode::BAD_REQUEST, "No valid fields provided").into_response();
    }

    let repo = UserRepository::new(&state.db);
    match repo.update_profile(id, update).await {
        Ok(Some(user)) => Json(UserResponse::from(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            tracing::error!("update_profile db error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/users/password",
    request_body = UpdatePasswordDto,
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 400, description = "Invalid input or passwords do not match")
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_password(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<UpdatePasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    if payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters".to_string(),
        ));
    }
    if payload.password != payload.change_password {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }

    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let hashed = hash_password(&payload.password);

    let repo = UserRepository::new(&state.db);
    repo.update_password(id, &hashed).await?;
    Ok((StatusCode::OK, "Password updated"))
}

#[utoipa::path(
    get,
    path = "/v1/profile",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    user_ctx: UserContext,
) -> impl IntoResponse {
    let id = match ObjectId::parse_str(&user_ctx.user_id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid user id").into_response(),
    };
    let repo = UserRepository::new(&state.db);
    match repo.find_by_id(id).await {
        Ok(Some(user)) => Json(UserResponse::from(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            tracing::error!("get_profile db error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
