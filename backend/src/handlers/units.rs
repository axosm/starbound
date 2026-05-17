use crate::auth::middleware::AuthPlayer;
use crate::errors::Result;
use crate::services::units::{self, RecruitRequest};
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn unit_defs(
    State(_state): State<Arc<AppState>>,
    AuthPlayer(_claims): AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    let defs: Vec<_> = units::UNIT_DEFS.iter().map(|d| serde_json::json!({
        "id": d.id,
        "base_hp": d.base_hp,
        "requires": d.requires,
        "cost": { "wood": d.wood_cost, "stone": d.stone_cost,
                  "food": d.food_cost, "iron": d.iron_cost },
    })).collect();
    Ok(Json(serde_json::json!({ "unit_defs": defs })))
}

pub async fn recruit(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<RecruitRequest>,
) -> Result<Json<serde_json::Value>> {
    let resp = units::recruit(&state, claims.sub, req).await?;
    Ok(Json(serde_json::json!({ "unit": resp })))
}
