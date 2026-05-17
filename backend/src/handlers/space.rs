use crate::auth::middleware::AuthPlayer;
use crate::errors::Result;
use crate::services::space::{self, CloakRequest, ScanRequest};
use crate::state::AppState;
use axum::{extract::{Query, State}, Json};
use std::sync::Arc;
use serde::Deserialize;

pub async fn scan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<ScanRequest>,
) -> Result<Json<serde_json::Value>> {
    let results = space::scan(&state, claims.sub, req).await?;
    Ok(Json(serde_json::json!({ "units": results })))
}

pub async fn set_cloak(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Json(req): Json<CloakRequest>,
) -> Result<Json<serde_json::Value>> {
    space::set_cloak(&state, claims.sub, req).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn battle_reports(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
    Query(q): Query<ReportsQuery>,
) -> Result<Json<serde_json::Value>> {
    let limit = q.limit.unwrap_or(20).min(100);
    let reports = space::battle_reports(&state, claims.sub, limit).await?;
    Ok(Json(serde_json::json!({ "reports": reports })))
}

pub async fn mark_reports_read(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    space::mark_reports_read(&state, claims.sub).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct ReportsQuery {
    pub limit: Option<i64>,
}
