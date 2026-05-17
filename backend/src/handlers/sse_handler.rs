/// Server-Sent Events handler.
///
/// The frontend connects here once and receives:
///   - "tick"          — every game tick (for countdown resync)
///   - "speed_changed" — when game speed changes
///   - "battle_started"— when an enemy arrives on one of the player's tiles
///   - "battle_ended"  — when a battle resolves
///
/// For everything else (construction complete, fleet arrived) the frontend
/// polls the relevant REST endpoints after their local timer fires.

use crate::auth::service::verify_token;
use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Deserialize)]
pub struct SseQuery {
    pub token: String,
}

pub async fn sse_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SseQuery>,
) -> impl IntoResponse {
    // Auth via query param (SSE cannot set custom headers in EventSource API)
    let claims = match verify_token(&q.token, &state.cfg.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            return Err(AppError::Unauthorized(e.to_string()));
        }
    };

    let player_id = claims.sub;
    let (global_rx, player_rx) = state.sse.subscribe_player(player_id).await;

    let global_stream = BroadcastStream::new(global_rx).filter_map(|r| async move {
        r.ok().map(|ev| {
            Ok::<Event, std::convert::Infallible>(
                Event::default()
                    .event(&ev.event_type)
                    .data(ev.payload.to_string()),
            )
        })
    });

    let player_stream = BroadcastStream::new(player_rx).filter_map(|r| async move {
        r.ok().map(|ev| {
            Ok::<Event, std::convert::Infallible>(
                Event::default()
                    .event(&ev.event_type)
                    .data(ev.payload.to_string()),
            )
        })
    });

    // Merge both streams
    let merged = stream::select(global_stream, player_stream);

    Ok(Sse::new(merged).keep_alive(KeepAlive::default()))
}

/// GET /api/events/missed?since_id=N
/// Returns SSE events the player may have missed (e.g. after page reload).
pub async fn missed_events(
    State(state): State<Arc<AppState>>,
    crate::auth::middleware::AuthPlayer(claims): crate::auth::middleware::AuthPlayer,
    Query(q): Query<MissedQuery>,
) -> crate::errors::Result<axum::Json<serde_json::Value>> {
    let player_id = claims.sub;
    let since_id = q.since_id.unwrap_or(0);

    let rows = match &state.db {
        crate::state::DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"SELECT id, event_type, payload FROM sse_events
                   WHERE (player_id = ? OR player_id IS NULL) AND id > ?
                   ORDER BY id ASC LIMIT 100"#,
                player_id, since_id
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|r| serde_json::json!({
                "id": r.id, "event_type": r.event_type,
                "payload": r.payload
            }))
            .collect::<Vec<_>>()
        }
        crate::state::DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"SELECT id, event_type, payload::text as payload FROM sse_events
                   WHERE (player_id = $1 OR player_id IS NULL) AND id > $2
                   ORDER BY id ASC LIMIT 100"#,
                player_id, since_id
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|r| serde_json::json!({
                "id": r.id, "event_type": r.event_type,
                "payload": r.payload
            }))
            .collect::<Vec<_>>()
        }
    };

    Ok(axum::Json(serde_json::json!({ "events": rows })))
}

#[derive(Deserialize)]
pub struct MissedQuery {
    pub since_id: Option<i64>,
}
