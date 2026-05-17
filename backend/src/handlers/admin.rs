use crate::auth::middleware::AuthPlayer;
use crate::errors::Result;
use crate::game::map;
use crate::state::{AppState, DbPool};
use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

pub async fn server_status(
    State(state): State<Arc<AppState>>,
    _auth: AuthPlayer,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "tick":         state.current_tick(),
        "speed":        state.game_speed(),
        "real_tick_ms": state.real_tick_ms(),
        "game_tick_ms": state.cfg.tick_ms,
        "db_driver":    state.cfg.db_driver,
    })))
}

/// POST /api/admin/planets/:id/generate
/// Generates all tiles for a planet (dev helper — fills DB from procgen).
pub async fn generate_planet_tiles(
    State(state): State<Arc<AppState>>,
    _auth: AuthPlayer,
    Path(planet_id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    let (seed, subdivision) = match &state.db {
        DbPool::Sqlite(pool) => {
            let r = sqlx::query!(
                "SELECT seed, subdivision FROM planets WHERE id=?", planet_id
            ).fetch_one(pool).await?;
            (r.seed as u64, r.subdivision as i64)
        }
        DbPool::Postgres(pool) => {
            let r = sqlx::query!(
                "SELECT seed, subdivision FROM planets WHERE id=$1", planet_id
            ).fetch_one(pool).await?;
            (r.seed as u64, r.subdivision as i64)
        }
    };

    let num_faces = 5i64;   // simplified: 5 faces for a Goldberg ico
    let mut count = 0usize;

    for face in 0..num_faces {
        let tiles = map::generate_face(seed, face, subdivision);
        for t in &tiles {
            let rare = t.rare_deposit;
            let yq   = t.yield_quality;
            let tt   = t.tile_type;
            let f    = t.face;
            let u    = t.u;
            let v    = t.v;
            match &state.db {
                DbPool::Sqlite(pool) => {
                    sqlx::query!(
                        r#"INSERT OR IGNORE INTO planet_tiles
                           (planet_id,face,u,v,tile_type,yield_quality,rare_deposit)
                           VALUES(?,?,?,?,?,?,?)"#,
                        planet_id, f, u, v, tt, yq, rare
                    ).execute(pool).await?;
                }
                DbPool::Postgres(pool) => {
                    sqlx::query!(
                        r#"INSERT INTO planet_tiles
                           (planet_id,face,u,v,tile_type,yield_quality,rare_deposit)
                           VALUES($1,$2,$3,$4,$5,$6,$7)
                           ON CONFLICT DO NOTHING"#,
                        planet_id, f, u, v, tt, yq, rare
                    ).execute(pool).await?;
                }
            }
            count += 1;
        }
    }

    Ok(Json(serde_json::json!({ "generated": count, "planet_id": planet_id })))
}
