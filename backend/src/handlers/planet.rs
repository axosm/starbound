use crate::auth::middleware::AuthPlayer;
use crate::dto::planet::{BuildRequest, PlanetViewResponse};
use crate::errors::Result;
use crate::services::planet;
use crate::state::AppState;
use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

pub async fn get_planet(
    State(state): State<Arc<AppState>>,
    AuthPlayer(_claims): AuthPlayer,
    Path(planet_id): Path<i64>,
) -> Result<Json<PlanetViewResponse>> {
    Ok(Json(planet::get_planet_view(&state, planet_id).await?))
}

pub async fn build_on_tile(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Path(planet_id): Path<i64>,
    Json(req): Json<BuildRequest>,
) -> Result<Json<serde_json::Value>> {
    planet::queue_build(&state, claims.sub, planet_id, req).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
