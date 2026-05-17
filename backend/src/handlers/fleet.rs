use crate::auth::middleware::AuthPlayer;
use crate::dto::fleet::{MoveOrderResponse, MoveUnitRequest, UnitDto};
use crate::errors::Result;
use crate::services::fleet;
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn list_units(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<Vec<UnitDto>>> {
    Ok(Json(fleet::list_units(&state, claims.sub).await?))
}

pub async fn move_unit(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<MoveUnitRequest>,
) -> Result<Json<MoveOrderResponse>> {
    Ok(Json(fleet::issue_move_order(&state, claims.sub, req).await?))
}
