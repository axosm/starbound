use crate::errors::Result;
use crate::state::{AppState, DbPool};

pub async fn tick_research(state: &AppState, tick: u64) -> Result<()> {
    let tick_i = tick as i64;
    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"UPDATE player_research
                   SET level = level + 1, research_done_tick = NULL
                   WHERE research_done_tick IS NOT NULL AND research_done_tick <= ?"#,
                tick_i
            ).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"UPDATE player_research
                   SET level = level + 1, research_done_tick = NULL
                   WHERE research_done_tick IS NOT NULL AND research_done_tick <= $1"#,
                tick_i
            ).execute(pool).await?;
        }
    }
    Ok(())
}
