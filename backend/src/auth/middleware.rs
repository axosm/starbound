use crate::auth::service::{verify_token, Claims};
use crate::errors::AppError;
use crate::state::AppState;
use axum::{async_trait, extract::FromRequestParts, http::{header, request::Parts}};
use std::sync::Arc;

pub struct AuthPlayer(pub Claims);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthPlayer {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> std::result::Result<Self, AppError> {
        let auth = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        let token = auth
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Bearer prefix missing".into()))?;

        let claims = verify_token(token, &state.cfg.jwt_secret)?;
        Ok(AuthPlayer(claims))
    }
}
