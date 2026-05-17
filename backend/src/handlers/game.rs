use crate::auth::middleware::AuthPlayer;
use crate::dto::game::GameInitResponse;
use crate::errors::Result;
use crate::services;
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn game_init(
    State(state): State<Arc<AppState>>,
    AuthPlayer(claims): AuthPlayer,
) -> Result<Json<GameInitResponse>> {
    let resp = services::game::init_player(&state, claims.sub, &claims.username).await?;
    Ok(Json(resp))
}

use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SystemQuery { pub galaxy_id: Option<i64> }

/// GET /api/systems?galaxy_id=N — list star systems (for solar system view)
pub async fn list_systems(
    State(state): State<Arc<AppState>>,
    AuthPlayer(_claims): AuthPlayer,
    Query(q): Query<SystemQuery>,
) -> crate::errors::Result<Json<serde_json::Value>> {
    let systems = match &state.db {
        crate::state::DbPool::Sqlite(pool) => {
            let gid = q.galaxy_id.unwrap_or(1);
            sqlx::query!(
                "SELECT id, seed, x, y, z FROM star_systems WHERE galaxy_id=? LIMIT 100", gid
            ).fetch_all(pool).await?
             .into_iter()
             .map(|r| serde_json::json!({
                 "id": r.id, "seed": r.seed, "x": r.x, "y": r.y, "z": r.z
             }))
             .collect::<Vec<_>>()
        }
        crate::state::DbPool::Postgres(pool) => {
            let gid = q.galaxy_id.unwrap_or(1);
            sqlx::query!(
                "SELECT id, seed, ST_X(pos) as x, ST_Y(pos) as y, ST_Z(pos) as z
                 FROM star_systems WHERE galaxy_id=$1 LIMIT 100", gid
            ).fetch_all(pool).await?
             .into_iter()
             .map(|r| serde_json::json!({
                 "id": r.id, "seed": r.seed,
                 "x": r.x.unwrap_or(0.0),
                 "y": r.y.unwrap_or(0.0),
                 "z": r.z.unwrap_or(0.0)
             }))
             .collect::<Vec<_>>()
        }
    };
    Ok(Json(serde_json::json!({ "systems": systems })))
}

/// GET /api/systems/:id/planets — list planets in a star system
pub async fn list_system_planets(
    State(state): State<Arc<AppState>>,
    AuthPlayer(_claims): AuthPlayer,
    axum::extract::Path(system_id): axum::extract::Path<i64>,
) -> crate::errors::Result<Json<serde_json::Value>> {
    let planets = match &state.db {
        crate::state::DbPool::Sqlite(pool) => {
            sqlx::query!(
                "SELECT id, seed, x, y, subdivision, planet_type FROM planets WHERE star_system_id=?",
                system_id
            ).fetch_all(pool).await?
             .into_iter()
             .map(|r| serde_json::json!({
                 "id": r.id, "star_system_id": system_id,
                 "seed": r.seed, "x": r.x, "y": r.y,
                 "subdivision": r.subdivision, "planet_type": r.planet_type
             }))
             .collect::<Vec<_>>()
        }
        crate::state::DbPool::Postgres(pool) => {
            sqlx::query!(
                "SELECT id, seed, ST_X(pos) as x, ST_Y(pos) as y, subdivision, planet_type
                 FROM planets WHERE star_system_id=$1",
                system_id
            ).fetch_all(pool).await?
             .into_iter()
             .map(|r| serde_json::json!({
                 "id": r.id, "star_system_id": system_id,
                 "seed": r.seed,
                 "x": r.x.unwrap_or(0.0),
                 "y": r.y.unwrap_or(0.0),
                 "subdivision": r.subdivision, "planet_type": r.planet_type
             }))
             .collect::<Vec<_>>()
        }
    };
    Ok(Json(serde_json::json!({ "planets": planets })))
}
