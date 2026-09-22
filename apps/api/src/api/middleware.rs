use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
    pub purpose: TokenPurpose,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_version: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_version: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenPurpose {
    Access,
    Download,
    RefreshCookie,
}

#[derive(Clone, Debug, Serialize, utoipa::ToSchema)]
pub struct UserContext {
    pub user_id: String,
}

impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<UserContext>()
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;

        Ok(user.clone())
    }
}

pub fn create_token(
    secret: &str,
    user_id: &str,
    purpose: TokenPurpose,
    file_id: Option<String>,
    exp: usize,
    share_version: Option<i32>,
    token_version: Option<i32>,
) -> Result<String, crate::api::error::AppError> {
    create_token_with_jti(
        secret,
        user_id,
        purpose,
        file_id,
        exp,
        share_version,
        token_version,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_token_with_jti(
    secret: &str,
    user_id: &str,
    purpose: TokenPurpose,
    file_id: Option<String>,
    exp: usize,
    share_version: Option<i32>,
    token_version: Option<i32>,
    jti: Option<String>,
) -> Result<String, crate::api::error::AppError> {
    let claims = Claims {
        user_id: user_id.to_string(),
        exp,
        purpose,
        file_id,
        share_version,
        token_version,
        jti,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        crate::api::error::AppError::InternalError(format!("Token encoding failed: {}", e))
    })
}

use std::sync::LazyLock;
use std::time::Duration;
use moka::future::Cache;

pub static USER_VERSION_CACHE: LazyLock<Cache<String, i32>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(50_000)
        .time_to_live(Duration::from_secs(60))
        .build()
});

pub static SESSION_VALID_CACHE: LazyLock<Cache<String, bool>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(100_000)
        .time_to_live(Duration::from_secs(60))
        .build()
});

pub async fn invalidate_auth_cache_for_user(user_id: &str) {
    USER_VERSION_CACHE.invalidate(user_id).await;
    let prefix = format!("{}:", user_id);
    let _ = SESSION_VALID_CACHE.invalidate_entries_if(move |k, _| k.starts_with(&prefix));
}

pub async fn auth_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    if claims.purpose != TokenPurpose::Access {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token_version = claims.token_version.ok_or(StatusCode::UNAUTHORIZED)?;
    let jti = claims.jti.as_deref().ok_or(StatusCode::UNAUTHORIZED)?;

    // 1. Check user token_version with cache
    let db_version = if let Some(v) = USER_VERSION_CACHE.get(&claims.user_id).await {
        v
    } else {
        let repo = crate::infrastructure::db::user_repo::UserRepository::new(&state.db);
        let id = mongodb::bson::oid::ObjectId::parse_str(&claims.user_id)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        match repo.find_by_id(id).await {
            Ok(Some(u)) => {
                USER_VERSION_CACHE
                    .insert(claims.user_id.clone(), u.token_version)
                    .await;
                u.token_version
            }
            _ => return Err(StatusCode::UNAUTHORIZED),
        }
    };

    if db_version != token_version {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 2. Check session validity with cache
    let session_key = format!("{}:{}:{}", claims.user_id, jti, token_version);
    let session_valid = if let Some(valid) = SESSION_VALID_CACHE.get(&session_key).await {
        valid
    } else {
        let sessions =
            crate::infrastructure::db::refresh_session_repo::RefreshSessionRepository::new(&state.db);
        let valid = matches!(
            sessions.exists(&claims.user_id, jti, token_version).await,
            Ok(true)
        );
        if valid {
            SESSION_VALID_CACHE.insert(session_key, true).await;
        }
        valid
    };

    if !session_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    req.extensions_mut().insert(UserContext {
        user_id: claims.user_id,
    });

    Ok(next.run(req).await)
}
