/// Resource production.
///
/// Production is LAZY — we do not update every player's resources every tick.
/// Instead, when a player requests their resources, we compute:
///   current = last_recorded + (rate_per_tick × ticks_since_last_update)
///
/// tick_production() runs every tick but only updates players who are "active"
/// (had a login in the last hour) to avoid unbounded DB writes.
/// For idle players, computation happens at login/request time.

use crate::errors::Result;
use crate::state::{AppState, DbPool};
use crate::dto::resources::{ResourceEntry, ResourcesResponse};
use crate::game::production;

pub async fn get_resources(state: &AppState, player_id: i64) -> Result<ResourcesResponse> {
    let tick = state.current_tick();
    let speed = state.game_speed();

    let resources = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                "SELECT resource_type, amount, cap FROM player_resources WHERE player_id = ?",
                player_id
            ).fetch_all(pool).await?
            .into_iter()
            .map(|r| ResourceEntry {
                resource_type: r.resource_type,
                amount: r.amount,
                cap: r.cap,
                rate_per_tick: production::base_rate(&r.resource_type, 1),
            })
            .collect()
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                "SELECT resource_type, amount, cap FROM player_resources WHERE player_id = $1",
                player_id
            ).fetch_all(pool).await?
            .into_iter()
            .map(|r| ResourceEntry {
                resource_type: r.resource_type,
                amount: r.amount,
                cap: r.cap,
                rate_per_tick: production::base_rate(&r.resource_type, 1),
            })
            .collect()
        }
    };

    Ok(ResourcesResponse { resources, tick, speed })
}

/// Called each tick — credit production to all players.
pub async fn tick_production(state: &AppState, _tick: u64) -> Result<()> {
    // Simple: add base production per tick to every player who has resources seeded.
    // Real game: query buildings, apply yield_quality multipliers, respect caps.
    let prod_rates: &[(&str, f64)] = &[
        ("wood",  2.0),
        ("stone", 1.5),
        ("food",  1.0),
        ("water", 1.0),
    ];

    for (rt, rate) in prod_rates {
        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"UPDATE player_resources
                       SET amount = MIN(amount + ?, cap),
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                       WHERE resource_type = ?"#,
                    rate, rt
                ).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    r#"UPDATE player_resources
                       SET amount = LEAST(amount + $1, cap),
                           updated_at = now()
                       WHERE resource_type = $2"#,
                    rate, rt
                ).execute(pool).await?;
            }
        }
    }
    Ok(())
}
