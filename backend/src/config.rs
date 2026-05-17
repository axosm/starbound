use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// "sqlite" | "postgres"
    #[serde(default = "default_db_driver")]
    pub db_driver: String,

    /// Full connection string
    pub database_url: String,

    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    /// Base game tick in real milliseconds at x1 speed (default: 60 000 = 1 min)
    #[serde(default = "default_tick_ms")]
    pub tick_ms: u64,

    /// Initial speed multiplier (1 | 2 | 5 | 10 | 100)
    #[serde(default = "default_game_speed")]
    pub game_speed: u64,

    /// Whether players (not just admin) can change speed via the API.
    /// Set to false on public servers.
    #[serde(default = "default_allow_player_speed")]
    pub allow_player_speed_change: bool,

    #[serde(default = "default_cors_origin")]
    pub cors_origin: String,
}

fn default_db_driver()          -> String { "sqlite".into() }
fn default_jwt_secret()         -> String { "change-me-in-production-secret-key-32ch".into() }
fn default_host()               -> String { "127.0.0.1".into() }
fn default_port()               -> u16    { 3000 }
fn default_tick_ms()            -> u64    { 60_000 }
fn default_game_speed()         -> u64    { 1 }
fn default_allow_player_speed() -> bool   { true }
fn default_cors_origin()        -> String { "http://localhost:5173".into() }

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        envy::from_env::<Self>().context("Failed to load config from environment")
    }

    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn real_tick_ms(&self, speed: u64) -> u64 {
        self.tick_ms / speed.max(1)
    }

    pub fn is_valid_speed(s: u64) -> bool {
        matches!(s, 1 | 2 | 5 | 10 | 100)
    }
}
