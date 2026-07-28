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
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenPurpose {
    Access,
    Download,
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
) -> Result<String, crate::api::error::AppError> {
    let claims = Claims {
        user_id: user_id.to_string(),
        exp,
        purpose,
        file_id,
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

    req.extensions_mut().insert(UserContext {
        user_id: claims.user_id,
    });

    Ok(next.run(req).await)
}
