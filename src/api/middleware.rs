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
pub struct Claims {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub exp: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserContext {
    pub user_id: String,
}

#[axum::async_trait]
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

pub async fn auth_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Some routes passed token in query param for downloads (token=...)
    let query_token = req.uri().query().and_then(|q| {
        q.split('&')
            .find(|pair| pair.starts_with("token="))
            .map(|pair| &pair[6..])
    });

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match query_token.or(auth_header) {
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

    req.extensions_mut().insert(UserContext {
        user_id: claims.user_id,
    });

    Ok(next.run(req).await)
}
