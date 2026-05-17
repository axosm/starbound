/// Server-Sent Events bus.
///
/// Architecture:
///   - Each connected SSE client gets a tokio broadcast receiver.
///   - Events are also persisted to `sse_events` table so polling clients
///     (or reconnecting SSE clients) can fetch missed events.
///   - Cleanup task removes expired events every 5 minutes.

use crate::state::AppState;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};

const CAPACITY: usize = 128;

#[derive(Clone, Debug, Serialize)]
pub struct SseEvent {
    pub event_type: String,
    pub player_id:  Option<i64>,   // None = broadcast to all
    pub payload:    serde_json::Value,
}

#[derive(Clone)]
pub struct SseBus {
    /// Global broadcast (every SSE client receives this)
    global: broadcast::Sender<SseEvent>,
    /// Per-player channels
    players: Arc<RwLock<HashMap<i64, broadcast::Sender<SseEvent>>>>,
}

impl SseBus {
    pub fn new() -> Self {
        let (global, _) = broadcast::channel(CAPACITY);
        Self { global, players: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn subscribe_player(
        &self,
        player_id: i64,
    ) -> (broadcast::Receiver<SseEvent>, broadcast::Receiver<SseEvent>) {
        let global_rx = self.global.subscribe();
        let mut map = self.players.write().await;
        let tx = map.entry(player_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(CAPACITY);
            tx
        });
        (global_rx, tx.subscribe())
    }

    /// Send event to a specific player.
    pub async fn send_to(&self, player_id: i64, event_type: &str, payload: serde_json::Value) {
        let ev = SseEvent {
            event_type: event_type.to_string(),
            player_id: Some(player_id),
            payload,
        };
        let map = self.players.read().await;
        if let Some(tx) = map.get(&player_id) {
            let _ = tx.send(ev);
        }
    }

    /// Broadcast to all connected clients.
    pub fn broadcast(&self, event_type: &str, payload: serde_json::Value) {
        let ev = SseEvent {
            event_type: event_type.to_string(),
            player_id: None,
            payload,
        };
        let _ = self.global.send(ev);
    }
}

/// Persist an SSE event to the DB so polling clients can catch up.
pub async fn persist_event(
    state: &AppState,
    player_id: Option<i64>,
    event_type: &str,
    payload: &serde_json::Value,
) -> anyhow::Result<()> {
    let payload_str = payload.to_string();
    // Expires in 24 hours
    match &state.db {
        crate::state::DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"INSERT INTO sse_events(player_id, event_type, payload, expires_at)
                   VALUES(?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '+1 day'))"#,
                player_id, event_type, payload_str
            ).execute(pool).await?;
        }
        crate::state::DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"INSERT INTO sse_events(player_id, event_type, payload, expires_at)
                   VALUES($1, $2, $3::jsonb, now() + interval '1 day')"#,
                player_id, event_type, payload_str
            ).execute(pool).await?;
        }
    }
    Ok(())
}

/// Background task: delete expired SSE events every 5 minutes.
pub async fn cleanup_loop(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(300));
    loop {
        ticker.tick().await;
        let result = match &state.db {
            crate::state::DbPool::Sqlite(pool) => {
                sqlx::query!(
                    "DELETE FROM sse_events WHERE expires_at < strftime('%Y-%m-%dT%H:%M:%fZ','now')"
                ).execute(pool).await.map(|r| r.rows_affected())
            }
            crate::state::DbPool::Postgres(pool) => {
                sqlx::query!(
                    "DELETE FROM sse_events WHERE expires_at < now()"
                ).execute(pool).await.map(|r| r.rows_affected())
            }
        };
        match result {
            Ok(n)  => tracing::debug!("SSE cleanup: removed {n} expired events"),
            Err(e) => tracing::warn!("SSE cleanup error: {e}"),
        }
    }
}
