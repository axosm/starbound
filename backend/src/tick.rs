/// Game tick loop.
///
/// Speed multiplier behaviour:
///   - `real_tick_ms = base_tick_ms / game_speed`
///   - All timers in the DB are stored in GAME TICKS (not wall-clock ms).
///   - Changing speed changes only how fast real time elapses per tick.
///   - Influence recalculation runs every 10 ticks (costly but not every tick).

use crate::services;
use crate::state::AppState;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::time::{interval, Duration, MissedTickBehavior};

pub async fn run_tick_loop(state: Arc<AppState>) {
    tracing::info!("Tick loop started");
    loop {
        let current_speed = state.game_speed();
        let tick_ms = state.cfg.real_tick_ms(current_speed);
        let mut timer = interval(Duration::from_millis(tick_ms));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            timer.tick().await;

            if state.game_speed() != current_speed {
                tracing::info!(
                    "Speed changed x{current_speed} → x{} — rebuilding timer",
                    state.game_speed()
                );
                break;
            }

            let tick = state.tick_count.fetch_add(1, Ordering::Relaxed) + 1;
            tracing::debug!("TICK {tick} (speed=x{current_speed})");

            // Persist tick counter every 100 ticks
            if tick % 100 == 0 {
                let s = Arc::clone(&state);
                let t = tick;
                tokio::spawn(async move {
                    let _ = s.config_set("game_tick", &t.to_string()).await;
                });
            }

            let s = Arc::clone(&state);
            tokio::spawn(async move { process_tick(s, tick).await });
        }
    }
}

async fn process_tick(state: Arc<AppState>, tick: u64) {
    // 1. Resolve unit/building arrivals → may trigger battles
    if let Err(e) = services::fleet::resolve_arrivals(&state, tick).await {
        tracing::error!("Tick {tick} fleet: {e}");
    }
    // 2. Battle rounds (one round per tick)
    if let Err(e) = services::battle::tick_battles(&state, tick).await {
        tracing::error!("Tick {tick} battle: {e}");
    }
    // 3. Passive resource production
    if let Err(e) = services::resources::tick_production(&state, tick).await {
        tracing::error!("Tick {tick} resources: {e}");
    }
    // 4. Complete construction
    if let Err(e) = services::construction::tick_construction(&state, tick).await {
        tracing::error!("Tick {tick} construction: {e}");
    }
    // 5. Complete research
    if let Err(e) = services::research::tick_research(&state, tick).await {
        tracing::error!("Tick {tick} research: {e}");
    }
    // 6. Influence recalculation every 10 ticks
    if tick % 10 == 0 {
        if let Err(e) = services::influence::recalc_flagged_tiles(&state).await {
            tracing::error!("Tick {tick} influence: {e}");
        }
    }
    // 7. Broadcast tick to all SSE clients
    state.sse.broadcast(
        "tick",
        serde_json::json!({
            "tick":         tick,
            "speed":        state.game_speed(),
            "real_tick_ms": state.real_tick_ms(),
        }),
    );
}
