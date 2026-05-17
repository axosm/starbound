use crate::auth::middleware::AuthPlayer;
use crate::errors::{AppError, Result};
use crate::services::alliance::{self, CreateEmpireRequest, InviteRequest, SetRoleRequest};
use crate::state::AppState;
use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

pub async fn get_my_empire(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    let emp = alliance::my_empire(&state, claims.sub).await?;
    Ok(Json(serde_json::json!({ "empire": emp })))
}

pub async fn create_empire(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<CreateEmpireRequest>,
) -> Result<Json<serde_json::Value>> {
    let emp = alliance::create_empire(&state, claims.sub, &req.name).await?;
    Ok(Json(serde_json::json!({ "empire": emp })))
}

pub async fn invite(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Path(empire_id): Path<i64>,
    Json(req): Json<InviteRequest>,
) -> Result<Json<serde_json::Value>> {
    alliance::invite_player(&state, empire_id, claims.sub, &req.target_username).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn kick(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Path((empire_id, target_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>> {
    alliance::kick_player(&state, empire_id, claims.sub, target_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn leave(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    alliance::leave_empire(&state, claims.sub).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn set_role(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Path(empire_id): Path<i64>,
    Json(req): Json<SetRoleRequest>,
) -> Result<Json<serde_json::Value>> {
    alliance::set_role(&state, empire_id, claims.sub, req.target_player_id, &req.role).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
