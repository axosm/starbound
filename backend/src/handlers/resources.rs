use crate::auth::middleware::AuthPlayer;
use crate::dto::resources::ResourcesResponse;
use crate::errors::Result;
use crate::services::resources;
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn get_resources(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<ResourcesResponse>> {
    Ok(Json(resources::get_resources(&state, claims.sub).await?))
}
