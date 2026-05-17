use crate::errors::Result;
use crate::state::{AppState, DbPool};

pub async fn tick_construction(state: &AppState, tick: u64) -> Result<()> {
    let tick_i = tick as i64;
    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"UPDATE buildings
                   SET construction_done_tick = NULL,
                       updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                   WHERE construction_done_tick IS NOT NULL AND construction_done_tick <= ?"#,
                tick_i
            ).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"UPDATE buildings
                   SET construction_done_tick = NULL, updated_at = now()
                   WHERE construction_done_tick IS NOT NULL AND construction_done_tick <= $1"#,
                tick_i
            ).execute(pool).await?;
        }
    }
    Ok(())
}
