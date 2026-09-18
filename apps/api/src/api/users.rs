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

use crate::api::auth_security::is_valid_email;

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
    #[serde(rename = "captchaId")]
    pub captcha_id: Option<String>,
    #[serde(rename = "captchaCode")]
    pub captcha_code: Option<String>,
    #[serde(rename = "emailCode")]
    pub email_code: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthConfigResponse {
    #[serde(rename = "allowRegistration")]
    pub allow_registration: bool,
    #[serde(rename = "emailVerificationRequired")]
    pub email_verification_required: bool,
    #[serde(rename = "captchaRequired")]
    pub captcha_required: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CaptchaResponse {
    pub id: String,
    pub svg: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SendEmailCodeDto {
    pub email: String,
    pub purpose: String,
    #[serde(rename = "captchaId")]
    pub captcha_id: String,
    #[serde(rename = "captchaCode")]
    pub captcha_code: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordDto {
    pub email: String,
    pub code: String,
    pub password: String,
    #[serde(rename = "changePassword")]
    pub change_password: String,
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
    #[serde(rename = "oldPassword")]
    pub old_password: String,
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
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || payload.password.is_empty() || payload.password.len() > 72 {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let repo = UserRepository::new(&state.db);
    let user = match repo.find_by_email(&email).await? {
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
    get,
    path = "/v1/auth/config",
    responses(
        (status = 200, description = "Authentication configuration", body = AuthConfigResponse)
    ),
    tag = "auth"
)]
pub async fn get_auth_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(AuthConfigResponse {
        allow_registration: state.config.allow_registration,
        email_verification_required: state.config.email_verification_required,
        captcha_required: state.config.captcha_required,
    })
}

#[utoipa::path(
    get,
    path = "/v1/auth/captcha",
    responses(
        (status = 200, description = "Generated captcha", body = CaptchaResponse)
    ),
    tag = "auth"
)]
pub async fn get_captcha(State(state): State<AppState>) -> impl IntoResponse {
    let (id, svg) = state.auth_security.generate_captcha();
    Json(CaptchaResponse { id, svg })
}

#[utoipa::path(
    post,
    path = "/v1/auth/send-code",
    request_body = SendEmailCodeDto,
    responses(
        (status = 200, description = "Verification code sent"),
        (status = 400, description = "Invalid captcha or email"),
        (status = 403, description = "Registration disabled"),
        (status = 404, description = "User not found")
    ),
    tag = "auth"
)]
pub async fn send_email_code(
    State(state): State<AppState>,
    Json(payload): Json<SendEmailCodeDto>,
) -> Result<impl IntoResponse, AppError> {
    if !state
        .auth_security
        .verify_and_consume_captcha(&payload.captcha_id, &payload.captcha_code)
    {
        return Err(AppError::BadRequest(
            "Invalid or expired captcha".to_string(),
        ));
    }

    let email = payload.email.trim().to_lowercase();
    if !is_valid_email(&email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    let repo = UserRepository::new(&state.db);
    match payload.purpose.as_str() {
        "register" => {
            if !state.config.allow_registration {
                return Err(AppError::Forbidden(
                    "User registration is closed".to_string(),
                ));
            }
            if repo.find_by_email(&email).await?.is_some() {
                return Err(AppError::BadRequest("User already exists".to_string()));
            }
        }
        "reset_password" => {
            let user_exists = repo.find_by_email(&email).await?.is_some();
            if !user_exists {
                // Prevent user enumeration: return 200 without sending email if user does not exist
                return Ok((
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "message": "If this email is registered, a verification code has been sent"
                    })),
                ));
            }
        }
        _ => return Err(AppError::BadRequest("Invalid purpose".to_string())),
    }

    state
        .auth_security
        .check_and_record_email_rate_limit(&email)
        .map_err(AppError::TooManyRequests)?;

    let code = state
        .auth_security
        .generate_email_code(&payload.purpose, &email);
    state
        .auth_security
        .send_email_code(&state.config, &email, &payload.purpose, &code)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to send email: {}", e)))?;

    let message = if state.config.smtp_host.is_some() {
        "Verification code sent to your email"
    } else {
        "Verification code sent (Mock mode: check backend server console)"
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": message })),
    ))
}

#[utoipa::path(
    post,
    path = "/v1/auth/reset-password",
    request_body = ResetPasswordDto,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 400, description = "Invalid verification code or passwords do not match"),
        (status = 404, description = "User not found")
    ),
    tag = "auth"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    if payload.password.len() < 6 || payload.password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be between 6 and 72 characters".to_string(),
        ));
    }
    if payload.password != payload.change_password {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }
    let email = payload.email.trim().to_lowercase();
    if !state
        .auth_security
        .verify_and_consume_email_code("reset_password", &email, &payload.code)
    {
        return Err(AppError::BadRequest(
            "Invalid or expired verification code".to_string(),
        ));
    }

    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_email(&email)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    let user_id = user
        .id
        .ok_or_else(|| AppError::BadRequest("User ID missing".to_string()))?;

    let hashed = hash_password(&payload.password);
    repo.update_password(user_id, &hashed).await?;

    // Invalidate and delete all refresh sessions for this user
    RefreshSessionRepository::new(&state.db)
        .delete_all_for_user(&user_id.to_hex())
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Password reset successfully" })),
    ))
}

#[utoipa::path(
    post,
    path = "/v1/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = CreateUserResponse),
        (status = 400, description = "User already exists, or invalid body"),
        (status = 403, description = "Registration is disabled")
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Check if registration is allowed
    if !state.config.allow_registration {
        return Err(AppError::Forbidden(
            "User registration is closed".to_string(),
        ));
    }

    // 2. Validate email format
    let email = payload.email.trim().to_lowercase();
    if !is_valid_email(&email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // 3. Validate names
    let first_name = payload.first_name.trim();
    let last_name = payload.last_name.trim();
    if first_name.is_empty() || first_name.len() > 100 {
        return Err(AppError::BadRequest(
            "First name must be between 1 and 100 characters".to_string(),
        ));
    }
    if last_name.is_empty() || last_name.len() > 100 {
        return Err(AppError::BadRequest(
            "Last name must be between 1 and 100 characters".to_string(),
        ));
    }

    // 4. Validate password length
    if payload.password.len() < 6 || payload.password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be between 6 and 72 characters".to_string(),
        ));
    }

    // 5. Verification logic:
    // - If email verification is enabled, verify the email code (its issuance was guarded by captcha in /v1/auth/send-code).
    // - If email verification is disabled, verify graphic captcha directly to prevent automated registrations.
    if state.config.email_verification_required {
        let code = match &payload.email_code {
            Some(c) if !c.trim().is_empty() => c.trim(),
            _ => {
                return Err(AppError::BadRequest(
                    "Email verification code is required".to_string(),
                ));
            }
        };
        if !state
            .auth_security
            .verify_and_consume_email_code("register", &email, code)
        {
            return Err(AppError::BadRequest(
                "Invalid or expired email verification code".to_string(),
            ));
        }
    } else if state.config.captcha_required
        || payload.captcha_id.is_some()
        || payload.captcha_code.is_some()
    {
        let (cid, ccode) = match (&payload.captcha_id, &payload.captcha_code) {
            (Some(id), Some(code)) if !id.trim().is_empty() && !code.trim().is_empty() => {
                (id, code)
            }
            _ => return Err(AppError::BadRequest("Captcha is required".to_string())),
        };
        if !state.auth_security.verify_and_consume_captcha(cid, ccode) {
            return Err(AppError::BadRequest(
                "Invalid or expired captcha".to_string(),
            ));
        }
    }

    let repo = UserRepository::new(&state.db);

    // Check if user exists
    if repo.find_by_email(&email).await?.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    let user = User {
        id: None,
        email,
        password: hash_password(&payload.password),
        first_name: first_name.to_string(),
        last_name: last_name.to_string(),
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
        if av.len() > 1024 * 1024 {
            return (StatusCode::BAD_REQUEST, "Invalid avatar").into_response();
        }
        if av.trim().is_empty() {
            update.insert("avatar", Bson::Null);
        } else {
            update.insert("avatar", av);
        }
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
    if payload.old_password.is_empty() {
        return Err(AppError::BadRequest(
            "Current password is required".to_string(),
        ));
    }
    if payload.password.len() < 6 || payload.password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password must be between 6 and 72 characters".to_string(),
        ));
    }
    if payload.password != payload.change_password {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }

    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if !verify_password(&payload.old_password, &user.password) {
        return Err(AppError::BadRequest(
            "Incorrect current password".to_string(),
        ));
    }

    let hashed = hash_password(&payload.password);
    repo.update_password(id, &hashed).await?;

    // Invalidate and delete all refresh sessions for this user
    RefreshSessionRepository::new(&state.db)
        .delete_all_for_user(&user_ctx.user_id)
        .await?;

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
