use crate::api::AppState;
use crate::api::middleware::UserContext;
use crate::domain::user::{User, UserResponse};
use crate::infrastructure::db::user_repo::UserRepository;
use crate::utils::crypto::{hash_password, verify_password};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use mongodb::bson::{doc, oid::ObjectId, Bson, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
) -> impl IntoResponse {
    let repo = UserRepository::new(&state.db);
    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    };

    if !verify_password(&payload.password, &user.password) {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    }

    let claims = crate::api::middleware::Claims {
        user_id: user.id.unwrap().to_hex(),
        exp: (Utc::now().timestamp() + 3600 * 24 * 7) as usize, // 1 week
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .unwrap();

    Json(LoginResponse {
        access_token: token,
    })
    .into_response()
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
) -> impl IntoResponse {
    let repo = UserRepository::new(&state.db);

    // Check if user exists
    if let Ok(Some(_)) = repo.find_by_email(&payload.email).await {
        return (StatusCode::BAD_REQUEST, "User already exists").into_response();
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
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };

    match repo.create(user).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(CreateUserResponse { id: id.to_hex() }),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/v1/users",
    responses(
        (status = 200, description = "List all users", body = [UserResponse])
    ),
    tag = "users"
)]
pub async fn get_all_users(State(state): State<AppState>) -> impl IntoResponse {
    let repo = UserRepository::new(&state.db);
    match repo.find_all().await {
        Ok(users) => Json(users.into_iter().map(UserResponse::from).collect::<Vec<_>>()).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users").into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User database id")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn get_user(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let id = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID").into_response(),
    };

    let repo = UserRepository::new(&state.db);
    match repo.find_by_id(id).await {
        Ok(Some(user)) => Json(UserResponse::from(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/users/profile/{id}",
    params(
        ("id" = String, Path, description = "User database id")
    ),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> impl IntoResponse {
    let id = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID").into_response(),
    };

    let mut update = doc! { "updatedAt": Bson::DateTime(BsonDateTime::from_chrono(Utc::now())) };
    if let Some(f) = payload.first_name {
        update.insert("firstName", f);
    }
    if let Some(l) = payload.last_name {
        update.insert("lastName", l);
    }
    if let Some(g) = payload.gender {
        update.insert("gender", g);
    }
    if let Some(a) = payload.age {
        update.insert("age", a);
    }
    if let Some(av) = payload.avatar {
        update.insert("avatar", av);
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
) -> impl IntoResponse {
    if payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            "Password must be at least 6 characters",
        )
            .into_response();
    }
    if payload.password != payload.change_password {
        return (StatusCode::BAD_REQUEST, "Passwords do not match").into_response();
    }

    let id = ObjectId::parse_str(&user_ctx.user_id).unwrap();
    let hashed = hash_password(&payload.password);

    let repo = UserRepository::new(&state.db);
    match repo.update_password(id, &hashed).await {
        Ok(_) => (StatusCode::OK, "Password updated").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User database id")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let id = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID").into_response(),
    };

    let repo = UserRepository::new(&state.db);
    match repo.delete_one(id).await {
        Ok(Some(user)) => Json(UserResponse::from(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
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
