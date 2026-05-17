/// Influence recalculation service.
/// 
/// Called every N ticks (not every tick — expensive).
/// Only processes tiles flagged with influence_recalc_needed = 1.

use crate::errors::Result;
use crate::game::influence::{score_for, InfluenceSource};
use crate::state::{AppState, DbPool};

pub async fn recalc_flagged_tiles(state: &AppState) -> Result<()> {
    // Get flagged tile IDs
    let tile_ids: Vec<i64> = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            "SELECT id FROM planet_tiles WHERE influence_recalc_needed = 1 LIMIT 200"
        ).fetch_all(pool).await?,
        DbPool::Postgres(pool) => sqlx::query_scalar!(
            "SELECT id FROM planet_tiles WHERE influence_recalc_needed = TRUE LIMIT 200"
        ).fetch_all(pool).await?,
    };

    for tile_id in tile_ids {
        recalc_tile(state, tile_id).await?;
    }
    Ok(())
}

async fn recalc_tile(state: &AppState, tile_id: i64) -> Result<()> {
    // Get tile planet + coords
    let (planet_id, tile_u, tile_v) = match &state.db {
        DbPool::Sqlite(pool) => {
            let r = sqlx::query!(
                "SELECT planet_id, u, v FROM planet_tiles WHERE id=?", tile_id
            ).fetch_one(pool).await?;
            (r.planet_id, r.u, r.v)
        }
        DbPool::Postgres(pool) => {
            let r = sqlx::query!(
                "SELECT planet_id, u, v FROM planet_tiles WHERE id=$1", tile_id
            ).fetch_one(pool).await?;
            (r.planet_id, r.u, r.v)
        }
    };

    // Get all buildings on this planet with their owners and positions
    struct BldRow { player_id: i64, building_type: String, level: i64, u: i64, v: i64, face: i64 }
    let buildings: Vec<BldRow> = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_as!(BldRow,
            r#"SELECT b.player_id, b.building_type, b.level,
                      t.u, t.v, t.face
               FROM buildings b
               JOIN planet_tiles t ON t.id = b.tile_id
               WHERE t.planet_id = ?
               AND b.construction_done_tick IS NULL
               AND b.destroyed_at IS NULL"#,
            planet_id
        ).fetch_all(pool).await?,
        DbPool::Postgres(pool) => sqlx::query_as!(BldRow,
            r#"SELECT b.player_id, b.building_type, b.level,
                      t.u, t.v, t.face
               FROM buildings b
               JOIN planet_tiles t ON t.id = b.tile_id
               WHERE t.planet_id = $1
               AND b.construction_done_tick IS NULL
               AND b.destroyed_at IS NULL"#,
            planet_id
        ).fetch_all(pool).await?,
    };

    // Compute per-player scores
    let mut player_ids: Vec<i64> = buildings.iter().map(|b| b.player_id).collect();
    player_ids.dedup();
    player_ids.sort();

    let mut best_score  = 0.0f64;
    let mut best_player: Option<i64> = None;

    for &pid in &player_ids {
        let sources: Vec<InfluenceSource> = buildings.iter()
            .filter(|b| b.player_id == pid)
            .map(|b| InfluenceSource {
                face:  b.face,
                u:     b.u,
                v:     b.v,
                btype: b.building_type.clone(),
                level: b.level,
            })
            .collect();

        let score = score_for(tile_u, tile_v, &sources);

        // Upsert influence score
        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"INSERT INTO tile_influence(tile_id, player_id, score)
                       VALUES(?,?,?)
                       ON CONFLICT(tile_id, player_id) DO UPDATE SET score=excluded.score,
                       updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')"#,
                    tile_id, pid, score
                ).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO tile_influence(tile_id, player_id, score)
                       VALUES($1,$2,$3)
                       ON CONFLICT(tile_id,player_id) DO UPDATE
                       SET score=EXCLUDED.score, updated_at=now()"#,
                    tile_id, pid, score
                ).execute(pool).await?;
            }
        }

        if score > best_score {
            best_score  = score;
            best_player = Some(pid);
        }
    }

    // Update tile owner + clear flag
    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                "UPDATE planet_tiles SET owner_player_id=?, influence_recalc_needed=0 WHERE id=?",
                best_player, tile_id
            ).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                "UPDATE planet_tiles SET owner_player_id=$1, influence_recalc_needed=FALSE WHERE id=$2",
                best_player, tile_id
            ).execute(pool).await?;
        }
    }
    Ok(())
}
