use crate::auth::middleware::AuthPlayer;
use crate::errors::Result;
use crate::services::research_api::{self, StartResearchRequest};
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn list_research(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    let techs = research_api::list_research(&state, claims.sub).await?;
    Ok(Json(serde_json::json!({ "techs": techs })))
}

pub async fn start_research(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<StartResearchRequest>,
) -> Result<Json<serde_json::Value>> {
    let tech = research_api::start_research(&state, claims.sub, &req.tech_id).await?;
    Ok(Json(serde_json::json!({ "tech": tech })))
}
