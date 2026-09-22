use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use webauthn_rs::prelude::*;

use crate::api::AppState;
use crate::api::error::AppError;
use crate::api::users::{LoginResponse, issue_auth_tokens};
use crate::domain::user::PasskeyInfo;
use crate::infrastructure::db::{
    refresh_session_repo::RefreshSessionRepository, user_repo::UserRepository,
};
use crate::utils::crypto::verify_password_async;

#[derive(Debug, Deserialize, ToSchema)]
pub struct WebauthnLoginStartDto {
    pub email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebauthnLoginStartResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[schema(value_type = Object)]
    pub challenge: RequestChallengeResponse,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WebauthnLoginFinishDto {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[schema(value_type = Object)]
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebauthnRegisterStartResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[schema(value_type = Object)]
    pub challenge: CreationChallengeResponse,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WebauthnRegisterFinishDto {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[schema(value_type = Object)]
    pub credential: RegisterPublicKeyCredential,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfirmPasswordDto {
    pub password: String,
}

/// Start Touch ID / Passkey authentication ceremony
#[utoipa::path(
    post,
    path = "/v1/auth/webauthn/login-start",
    request_body = WebauthnLoginStartDto,
    responses(
        (status = 200, description = "Challenge generated", body = WebauthnLoginStartResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "User not found")
    ),
    tag = "auth"
)]
pub async fn login_start(
    State(state): State<AppState>,
    Json(payload): Json<WebauthnLoginStartDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = UserRepository::new(&state.db);
    let user = if let Some(ref email) = payload.email {
        let trimmed = email.trim();
        if !trimmed.is_empty() {
            repo.find_by_email(trimmed).await?
        } else {
            None
        }
    } else {
        None
    };

    let (challenge, session_id) = state.webauthn.start_authentication(user.as_ref()).await?;

    Ok(Json(WebauthnLoginStartResponse {
        session_id,
        challenge,
    }))
}

/// Finish Touch ID / Passkey authentication ceremony
#[utoipa::path(
    post,
    path = "/v1/auth/webauthn/login-finish",
    request_body = WebauthnLoginFinishDto,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Authentication failed")
    ),
    tag = "auth"
)]
pub async fn login_finish(
    State(state): State<AppState>,
    Json(payload): Json<WebauthnLoginFinishDto>,
) -> Result<impl IntoResponse, AppError> {
    let cred_id_str = payload.credential.id.to_string();
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_passkey_id(&cred_id_str)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Passkey not recognized".to_string()))?;

    let stored_pk = user
        .passkeys
        .iter()
        .find(|p| p.id == cred_id_str)
        .ok_or_else(|| AppError::Unauthorized("Passkey not found on user".to_string()))?;

    let passkey = stored_pk
        .get_passkey()
        .map_err(|e| AppError::InternalError(format!("Failed to parse stored passkey: {}", e)))?;

    let auth_result = state
        .webauthn
        .finish_authentication(&payload.session_id, &payload.credential, Some(&passkey))
        .await?;

    let user_id_obj = user
        .id
        .ok_or_else(|| AppError::InternalError("Missing user ID".to_string()))?;
    let user_id = user_id_obj.to_hex();

    // Persist counter/backup-state changes before granting a session.
    let mut pk = passkey;
    pk.update_credential(&auth_result)
        .ok_or_else(|| AppError::Unauthorized("Passkey mismatch".to_string()))?;
    let updated_json = serde_json::to_string(&pk)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize passkey: {e}")))?;
    if !repo
        .update_passkey(
            user_id_obj,
            &cred_id_str,
            &stored_pk.passkey_json,
            &updated_json,
        )
        .await?
    {
        return Err(AppError::Unauthorized(
            "Passkey changed during authentication".to_string(),
        ));
    }

    let tokens = issue_auth_tokens(&state.config.jwt_secret, &user_id, user.token_version)?;
    RefreshSessionRepository::new(&state.db)
        .create(
            &user_id,
            &tokens.refresh_jti,
            user.token_version,
            tokens.refresh_expires_at,
        )
        .await?;

    Ok((
        [(
            header::SET_COOKIE,
            super::users::refresh_cookie(&tokens.refresh_token),
        )],
        Json(LoginResponse {
            access_token: tokens.access_token,
        }),
    ))
}

/// List all passkeys / Touch ID credentials registered for current user
#[utoipa::path(
    get,
    path = "/v1/auth/webauthn/passkeys",
    responses(
        (status = 200, description = "List of registered passkeys", body = Vec<PasskeyInfo>)
    ),
    security(("Bearer" = [])),
    tag = "auth"
)]
pub async fn list_passkeys(
    State(state): State<AppState>,
    user_ctx: crate::api::middleware::UserContext,
) -> Result<impl IntoResponse, AppError> {
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let list: Vec<PasskeyInfo> = user.passkeys.iter().map(PasskeyInfo::from).collect();
    Ok(Json(list))
}

/// Start Touch ID / Passkey registration ceremony (requires authentication)
#[utoipa::path(
    post,
    path = "/v1/auth/webauthn/register-start",
    responses(
        (status = 200, description = "Registration challenge generated", body = WebauthnRegisterStartResponse)
    ),
    security(("Bearer" = [])),
    tag = "auth"
)]
pub async fn register_start(
    State(state): State<AppState>,
    user_ctx: crate::api::middleware::UserContext,
    Json(payload): Json<ConfirmPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    state
        .security_actions
        .check(&user_ctx.user_id)
        .map_err(|_| AppError::TooManyRequests("Too many security changes".to_string()))?;
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    if !verify_password_async(payload.password, user.password.clone()).await {
        return Err(AppError::Unauthorized("Incorrect password".to_string()));
    }

    let (challenge, session_id) = state.webauthn.start_registration(&user).await?;

    Ok(Json(WebauthnRegisterStartResponse {
        session_id,
        challenge,
    }))
}

/// Finish Touch ID / Passkey registration ceremony (requires authentication)
#[utoipa::path(
    post,
    path = "/v1/auth/webauthn/register-finish",
    request_body = WebauthnRegisterFinishDto,
    responses(
        (status = 200, description = "Passkey registered successfully", body = PasskeyInfo)
    ),
    security(("Bearer" = [])),
    tag = "auth"
)]
pub async fn register_finish(
    State(state): State<AppState>,
    user_ctx: crate::api::middleware::UserContext,
    Json(payload): Json<WebauthnRegisterFinishDto>,
) -> Result<impl IntoResponse, AppError> {
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    let _user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let stored_passkey = state
        .webauthn
        .finish_registration(
            &payload.session_id,
            &user_ctx.user_id,
            &payload.credential,
            payload.name,
        )
        .await?;

    if let Some(existing_user) = repo.find_by_passkey_id(&stored_passkey.id).await? {
        let msg = if existing_user.id == Some(id) {
            "This passkey is already registered on your account"
        } else {
            "This passkey is already registered on another account"
        };
        return Err(AppError::BadRequest(msg.to_string()));
    }

    let info = PasskeyInfo::from(&stored_passkey);
    repo.add_passkey(id, stored_passkey).await?;

    Ok(Json(info))
}

/// Delete a registered passkey / Touch ID credential
#[utoipa::path(
    delete,
    path = "/v1/auth/webauthn/passkeys/{id}",
    responses(
        (status = 204, description = "Passkey deleted")
    ),
    security(("Bearer" = [])),
    tag = "auth"
)]
pub async fn delete_passkey(
    State(state): State<AppState>,
    Path(passkey_id): Path<String>,
    user_ctx: crate::api::middleware::UserContext,
    Json(payload): Json<ConfirmPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    state
        .security_actions
        .check(&user_ctx.user_id)
        .map_err(|_| AppError::TooManyRequests("Too many security changes".to_string()))?;
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    if !verify_password_async(payload.password, user.password).await {
        return Err(AppError::Unauthorized("Incorrect password".to_string()));
    }
    if !repo.delete_passkey(id, &passkey_id).await? {
        return Err(AppError::NotFound("Passkey not found".to_string()));
    }
    RefreshSessionRepository::new(&state.db)
        .delete_all_for_user(&user_ctx.user_id)
        .await?;
    crate::api::middleware::invalidate_auth_cache_for_user(&user_ctx.user_id).await;

    Ok(StatusCode::NO_CONTENT)
}
