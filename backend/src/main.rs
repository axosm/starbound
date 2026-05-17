mod auth;
mod config;
mod db;
mod dto;
mod errors;
mod game;
mod handlers;
mod procgen;
mod routes;
mod services;
mod sse;
mod state;
mod tick;

use crate::config::AppConfig;
use crate::state::AppState;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "starbound=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let cfg = AppConfig::from_env()?;
    tracing::info!(
        "Starbound starting — db={} tick={}ms speed=x{}",
        cfg.db_driver,
        cfg.tick_ms,
        cfg.game_speed
    );

    let state = Arc::new(AppState::new(cfg).await?);
    state.run_migrations().await?;

    // Restore persisted speed from DB
    state.load_speed_from_db().await?;

    // Game tick loop
    let s = Arc::clone(&state);
    tokio::spawn(async move { tick::run_tick_loop(s).await });

    // SSE cleanup loop (purge expired events every 5 min)
    let s = Arc::clone(&state);
    tokio::spawn(async move { sse::cleanup_loop(s).await });

    let app = routes::build_router(Arc::clone(&state));
    let addr = state.cfg.listen_addr();
    tracing::info!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
