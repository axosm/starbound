use crate::auth::middleware::AuthPlayer;
use crate::config::AppConfig;
use crate::dto::speed::{SetSpeedRequest, SpeedResponse};
use crate::errors::{AppError, Result};
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn get_speed(State(state): State<Arc<AppState>>) -> Result<Json<SpeedResponse>> {
    Ok(Json(SpeedResponse {
        speed:                    state.game_speed(),
        real_tick_ms:             state.real_tick_ms(),
        game_tick_ms:             state.cfg.tick_ms,
        allow_player_speed_change: state.cfg.allow_player_speed_change,
        current_tick:             state.current_tick(),
    }))
}

pub async fn set_speed(
    State(state): State<Arc<AppState>>,
    _auth: AuthPlayer,
    Json(req): Json<SetSpeedRequest>,
) -> Result<Json<SpeedResponse>> {
    if !state.cfg.allow_player_speed_change {
        return Err(AppError::Forbidden(
            "Speed changes are disabled on this server".into(),
        ));
    }
    if !AppConfig::is_valid_speed(req.speed) {
        return Err(AppError::BadRequest(format!(
            "Invalid speed '{}'. Allowed: 1, 2, 5, 10, 100",
            req.speed
        )));
    }

    let old = state.game_speed();
    state
        .set_speed(req.speed)
        .await
        .map_err(AppError::Internal)?;
    tracing::info!("Game speed changed x{old} → x{}", req.speed);

    // Notify all SSE clients so their countdown timers resync
    state.sse.broadcast(
        "speed_changed",
        serde_json::json!({
            "speed": req.speed,
            "real_tick_ms": state.real_tick_ms(),
            "game_tick_ms": state.cfg.tick_ms,
            "current_tick": state.current_tick(),
        }),
    );

    Ok(Json(SpeedResponse {
        speed:                    state.game_speed(),
        real_tick_ms:             state.real_tick_ms(),
        game_tick_ms:             state.cfg.tick_ms,
        allow_player_speed_change: state.cfg.allow_player_speed_change,
        current_tick:             state.current_tick(),
    }))
}
