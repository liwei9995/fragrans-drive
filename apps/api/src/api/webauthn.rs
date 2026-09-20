use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use webauthn_rs::prelude::*;

use crate::api::error::AppError;
use crate::api::users::{issue_auth_tokens, LoginResponse};
use crate::api::AppState;
use crate::domain::user::PasskeyInfo;
use crate::infrastructure::db::{
    refresh_session_repo::RefreshSessionRepository, user_repo::UserRepository,
};

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

    let (challenge, session_id) = state
        .webauthn
        .start_authentication(user.as_ref())
        .await?;

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
    let auth_result = state
        .webauthn
        .finish_authentication(&payload.session_id, &payload.credential)
        .await?;

    let cred_id_str = payload.credential.id.to_string();
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_passkey_id(&cred_id_str)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Passkey not recognized".to_string()))?;

    let user_id_obj = user
        .id
        .ok_or_else(|| AppError::InternalError("Missing user ID".to_string()))?;
    let user_id = user_id_obj.to_hex();

    // Update the counter on the stored passkey to prevent replay attacks
    if let Some(stored_pk) = user.passkeys.into_iter().find(|p| p.id == cred_id_str) {
        if let Ok(mut pk) = stored_pk.get_passkey() {
            pk.update_credential(&auth_result);
            if let Ok(updated_json) = serde_json::to_string(&pk) {
                let _ = repo
                    .update_passkey(user_id_obj, &cred_id_str, &updated_json)
                    .await;
            }
        }
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

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }))
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
) -> Result<impl IntoResponse, AppError> {
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

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
        .finish_registration(&payload.session_id, &payload.credential, payload.name)
        .await?;

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
) -> Result<impl IntoResponse, AppError> {
    let id = ObjectId::parse_str(&user_ctx.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;
    let repo = UserRepository::new(&state.db);
    repo.delete_passkey(id, &passkey_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
