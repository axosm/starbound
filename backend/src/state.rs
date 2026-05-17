use crate::config::AppConfig;
use crate::sse::SseBus;
use anyhow::Context;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ─── DB pool enum ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum DbPool {
    Sqlite(sqlx::SqlitePool),
    Postgres(sqlx::PgPool),
}

// ─── App state ───────────────────────────────────────────────────────────────

pub struct AppState {
    pub cfg:        AppConfig,
    pub db:         DbPool,
    /// Runtime speed — changed atomically so tick loop picks it up instantly.
    pub speed:      Arc<AtomicU64>,
    pub tick_count: Arc<AtomicU64>,
    pub sse:        SseBus,
}

impl AppState {
    pub async fn new(cfg: AppConfig) -> anyhow::Result<Self> {
        let db = match cfg.db_driver.as_str() {
            "sqlite" => {
                let pool = sqlx::SqlitePool::connect(&cfg.database_url)
                    .await
                    .context("SQLite connect")?;
                DbPool::Sqlite(pool)
            }
            "postgres" => {
                let pool = sqlx::PgPool::connect(&cfg.database_url)
                    .await
                    .context("Postgres connect")?;
                DbPool::Postgres(pool)
            }
            other => anyhow::bail!("Unknown db_driver: {other}. Use 'sqlite' or 'postgres'"),
        };
        let speed = Arc::new(AtomicU64::new(cfg.game_speed));
        Ok(Self {
            speed,
            db,
            tick_count: Arc::new(AtomicU64::new(0)),
            sse: SseBus::new(),
            cfg,
        })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        match &self.db {
            DbPool::Sqlite(pool) => {
                sqlx::migrate!("./migrations/sqlite").run(pool).await
                    .context("SQLite migrations")?;
            }
            DbPool::Postgres(pool) => {
                sqlx::migrate!("./migrations/postgres").run(pool).await
                    .context("Postgres migrations")?;
            }
        }
        tracing::info!("Migrations applied");
        Ok(())
    }

    /// Read persisted speed from server_config table.
    pub async fn load_speed_from_db(&self) -> anyhow::Result<()> {
        let speed_str = self.config_get("game_speed").await?.unwrap_or("1".into());
        let tick_str  = self.config_get("game_tick").await?.unwrap_or("0".into());
        let speed: u64 = speed_str.parse().unwrap_or(1);
        let tick:  u64 = tick_str.parse().unwrap_or(0);
        self.speed.store(speed, Ordering::Relaxed);
        self.tick_count.store(tick, Ordering::Relaxed);
        tracing::info!("Restored: tick={tick} speed=x{speed}");
        Ok(())
    }

    pub async fn config_get(&self, key: &str) -> anyhow::Result<Option<String>> {
        match &self.db {
            DbPool::Sqlite(pool) => {
                let row = sqlx::query_scalar!(
                    "SELECT value FROM server_config WHERE key = ?", key
                ).fetch_optional(pool).await?;
                Ok(row)
            }
            DbPool::Postgres(pool) => {
                let row = sqlx::query_scalar!(
                    "SELECT value FROM server_config WHERE key = $1", key
                ).fetch_optional(pool).await?;
                Ok(row)
            }
        }
    }

    pub async fn config_set(&self, key: &str, value: &str) -> anyhow::Result<()> {
        match &self.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    "INSERT INTO server_config(key,value) VALUES(?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    key, value
                ).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    "INSERT INTO server_config(key,value) VALUES($1,$2) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value",
                    key, value
                ).execute(pool).await?;
            }
        }
        Ok(())
    }

    pub fn game_speed(&self) -> u64 { self.speed.load(Ordering::Relaxed) }
    pub fn real_tick_ms(&self) -> u64 { self.cfg.real_tick_ms(self.game_speed()) }
    pub fn current_tick(&self) -> u64 { self.tick_count.load(Ordering::Relaxed) }

    pub async fn set_speed(&self, speed: u64) -> anyhow::Result<()> {
        self.speed.store(speed, Ordering::Relaxed);
        self.config_set("game_speed", &speed.to_string()).await
    }
}
