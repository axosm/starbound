use crate::errors::Result;
use crate::game::combat;
use crate::state::{AppState, DbPool};

/// One combat round per tick for each active battle.
pub async fn tick_battles(state: &AppState, tick: u64) -> Result<()> {
    let tick_i = tick as i64;

    let battles = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                "SELECT id, tile_id, attacker_id, defender_id, phase FROM battles WHERE last_tick < ?",
                tick_i
            ).fetch_all(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                "SELECT id, tile_id, attacker_id, defender_id, phase FROM battles WHERE last_tick < $1",
                tick_i
            ).fetch_all(pool).await?
        }
    };

    for b in battles {
        run_battle_round(state, b.id, b.tile_id, b.attacker_id, b.defender_id, &b.phase, tick_i).await?;
    }
    Ok(())
}

async fn run_battle_round(
    state: &AppState,
    battle_id: i64,
    tile_id: Option<i64>,
    attacker_id: i64,
    defender_id: i64,
    phase: &str,
    tick: i64,
) -> Result<()> {
    // Gather attacker units on the tile
    struct UnitRow { id: i64, hp: i64, count: i64, unit_type: String }

    let (attacker_units, defender_units) = match &state.db {
        DbPool::Sqlite(pool) => {
            let a = sqlx::query_as!(UnitRow,
                "SELECT id, hp, count, unit_type FROM units WHERE player_id=? AND in_battle=1 AND planet_id IS NOT NULL",
                attacker_id
            ).fetch_all(pool).await?;
            let d = sqlx::query_as!(UnitRow,
                "SELECT id, hp, count, unit_type FROM units WHERE player_id=? AND in_battle=1 AND planet_id IS NOT NULL",
                defender_id
            ).fetch_all(pool).await?;
            (a, d)
        }
        DbPool::Postgres(pool) => {
            let a = sqlx::query_as!(UnitRow,
                "SELECT id, hp, count, unit_type FROM units WHERE player_id=$1 AND in_battle=true AND planet_id IS NOT NULL",
                attacker_id
            ).fetch_all(pool).await?;
            let d = sqlx::query_as!(UnitRow,
                "SELECT id, hp, count, unit_type FROM units WHERE player_id=$1 AND in_battle=true AND planet_id IS NOT NULL",
                defender_id
            ).fetch_all(pool).await?;
            (a, d)
        }
    };

    if attacker_units.is_empty() || defender_units.is_empty() {
        // Battle over — write report and clean up
        let outcome = if attacker_units.is_empty() { "defender_victory" } else { "attacker_victory" };
        let attacker_snap = serde_json::json!([]);
        let defender_snap = serde_json::json!([]);

        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"INSERT INTO battle_reports(battle_id,tile_id,attacker_id,defender_id,
                       outcome,attacker_units_snapshot,defender_units_snapshot,started_tick,ended_tick)
                       VALUES(?,?,?,?,?,?,?,?,?)"#,
                    battle_id, tile_id, attacker_id, defender_id,
                    outcome, attacker_snap.to_string(), defender_snap.to_string(),
                    tick, tick
                ).execute(pool).await?;
                sqlx::query!("DELETE FROM battles WHERE id=?", battle_id).execute(pool).await?;
                // Reset in_battle flags
                sqlx::query!("UPDATE units SET in_battle=0 WHERE player_id=? OR player_id=?",
                    attacker_id, defender_id).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO battle_reports(battle_id,tile_id,attacker_id,defender_id,
                       outcome,attacker_units_snapshot,defender_units_snapshot,started_tick,ended_tick)
                       VALUES($1,$2,$3,$4,$5,$6::jsonb,$7::jsonb,$8,$9)"#,
                    battle_id, tile_id, attacker_id, defender_id,
                    outcome, attacker_snap.to_string(), defender_snap.to_string(),
                    tick, tick
                ).execute(pool).await?;
                sqlx::query!("DELETE FROM battles WHERE id=$1", battle_id).execute(pool).await?;
                sqlx::query!("UPDATE units SET in_battle=false WHERE player_id=$1 OR player_id=$2",
                    attacker_id, defender_id).execute(pool).await?;
            }
        }

        // Alert players via SSE
        let payload = serde_json::json!({"battle_id": battle_id, "outcome": outcome});
        state.sse.send_to(attacker_id, "battle_ended", payload.clone()).await;
        state.sse.send_to(defender_id, "battle_ended", payload).await;
        return Ok(());
    }

    // Apply one round of damage to each side
    for unit in &attacker_units {
        let dmg = combat::unit_damage(&unit.unit_type, unit.count);
        // Spread damage across defenders equally
        let per_unit = dmg / defender_units.len().max(1) as i64;
        for target in &defender_units {
            let new_hp = (target.hp - per_unit).max(0);
            match &state.db {
                DbPool::Sqlite(pool) => {
                    sqlx::query!("UPDATE units SET hp=? WHERE id=?", new_hp, target.id)
                        .execute(pool).await?;
                    if new_hp == 0 {
                        sqlx::query!("DELETE FROM units WHERE id=?", target.id).execute(pool).await?;
                    }
                }
                DbPool::Postgres(pool) => {
                    sqlx::query!("UPDATE units SET hp=$1 WHERE id=$2", new_hp, target.id)
                        .execute(pool).await?;
                    if new_hp == 0 {
                        sqlx::query!("DELETE FROM units WHERE id=$1", target.id).execute(pool).await?;
                    }
                }
            }
        }
    }

    // Update last_tick
    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!("UPDATE battles SET last_tick=? WHERE id=?", tick, battle_id)
                .execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!("UPDATE battles SET last_tick=$1 WHERE id=$2", tick, battle_id)
                .execute(pool).await?;
        }
    }
    Ok(())
}
