use crate::dto::planet::{BuildRequest, BuildingDto, PlanetViewResponse, TileDto, UnitOnTileDto};
use crate::errors::{AppError, Result};
use crate::game::buildings;
use crate::state::{AppState, DbPool};

pub async fn get_planet_view(state: &AppState, planet_id: i64) -> Result<PlanetViewResponse> {
    let planet = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                "SELECT id, seed, subdivision, planet_type FROM planets WHERE id=?",
                planet_id
            ).fetch_optional(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                "SELECT id, seed, subdivision, planet_type FROM planets WHERE id=$1",
                planet_id
            ).fetch_optional(pool).await?
        }
    }.ok_or_else(|| AppError::NotFound(format!("Planet {planet_id} not found")))?;

    let tiles = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(r#"
                SELECT t.id, t.face, t.u, t.v, t.tile_type, t.yield_quality, t.rare_deposit, t.owner_player_id,
                       b.id as bld_id, b.building_type, b.level, b.hp as bld_hp, b.max_hp,
                       b.under_attack, b.construction_done_tick, b.flight_state
                FROM planet_tiles t
                LEFT JOIN buildings b ON b.tile_id = t.id
                WHERE t.planet_id = ?
                ORDER BY t.face, t.u, t.v
            "#, planet_id).fetch_all(pool).await?
            .into_iter()
            .map(|r| TileDto {
                id: r.id,
                face: r.face,
                u: r.u,
                v: r.v,
                tile_type: r.tile_type,
                yield_quality: r.yield_quality,
                rare_deposit: r.rare_deposit,
                owner_player_id: r.owner_player_id,
                building: r.bld_id.map(|bid| BuildingDto {
                    id: bid,
                    building_type: r.building_type.unwrap_or_default(),
                    level: r.level.unwrap_or(1),
                    hp: r.bld_hp.unwrap_or(0),
                    max_hp: r.max_hp.unwrap_or(0),
                    under_attack: r.under_attack.unwrap_or(0) != 0,
                    construction_done_tick: r.construction_done_tick,
                    flight_state: r.flight_state,
                }),
                units: vec![],
            })
            .collect::<Vec<_>>()
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(r#"
                SELECT t.id, t.face, t.u, t.v, t.tile_type, t.yield_quality, t.rare_deposit, t.owner_player_id,
                       b.id as bld_id, b.building_type, b.level, b.hp as bld_hp, b.max_hp,
                       b.under_attack, b.construction_done_tick, b.flight_state
                FROM planet_tiles t
                LEFT JOIN buildings b ON b.tile_id = t.id
                WHERE t.planet_id = $1
                ORDER BY t.face, t.u, t.v
            "#, planet_id).fetch_all(pool).await?
            .into_iter()
            .map(|r| TileDto {
                id: r.id,
                face: r.face as i64,
                u: r.u as i64,
                v: r.v as i64,
                tile_type: r.tile_type,
                yield_quality: r.yield_quality as f64,
                rare_deposit: r.rare_deposit,
                owner_player_id: r.owner_player_id,
                building: r.bld_id.map(|bid| BuildingDto {
                    id: bid,
                    building_type: r.building_type.unwrap_or_default(),
                    level: r.level.unwrap_or(1) as i64,
                    hp: r.bld_hp.unwrap_or(0) as i64,
                    max_hp: r.max_hp.unwrap_or(0) as i64,
                    under_attack: r.under_attack.unwrap_or(false),
                    construction_done_tick: r.construction_done_tick,
                    flight_state: r.flight_state,
                }),
                units: vec![],
            })
            .collect::<Vec<_>>()
        }
    };

    Ok(PlanetViewResponse {
        planet_id: planet.id,
        seed: planet.seed,
        subdivision: planet.subdivision as i64,
        planet_type: planet.planet_type,
        tiles,
    })
}

pub async fn queue_build(
    state: &AppState,
    player_id: i64,
    planet_id: i64,
    req: BuildRequest,
) -> Result<()> {
    // Find or create the tile
    let tile_id: Option<i64> = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "SELECT id FROM planet_tiles WHERE planet_id=? AND face=? AND u=? AND v=?",
                planet_id, req.face, req.u, req.v
            ).fetch_optional(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "SELECT id FROM planet_tiles WHERE planet_id=$1 AND face=$2 AND u=$3 AND v=$4",
                planet_id, req.face, req.u, req.v
            ).fetch_optional(pool).await?
        }
    };
    let tile_id = tile_id.ok_or_else(|| AppError::NotFound("Tile not found".into()))?;

    // Check ownership
    let owned = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM planet_tiles WHERE id=? AND owner_player_id=?",
                tile_id, player_id
            ).fetch_one(pool).await? > 0
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM planet_tiles WHERE id=$1 AND owner_player_id=$2",
                tile_id, player_id
            ).fetch_one(pool).await?.unwrap_or(0) > 0
        }
    };
    if !owned {
        return Err(AppError::Forbidden("You don't own this tile".into()));
    }

    // Check no existing building
    let has_bld = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM buildings WHERE tile_id=? AND destroyed_at IS NULL",
                tile_id
            ).fetch_one(pool).await? > 0
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM buildings WHERE tile_id=$1 AND destroyed_at IS NULL",
                tile_id
            ).fetch_one(pool).await?.unwrap_or(0) > 0
        }
    };
    if has_bld {
        return Err(AppError::Conflict("Tile already has a building".into()));
    }

    let def = buildings::get_def(&req.building_type)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown building type: {}", req.building_type)))?;

    let tick = state.current_tick() as i64;
    let done_tick = tick + def.build_ticks;

    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"INSERT INTO buildings(player_id, building_type, tile_id, hp, max_hp, construction_done_tick)
                   VALUES(?,?,?,?,?,?)"#,
                player_id, req.building_type, tile_id,
                def.base_hp, def.base_hp, done_tick
            ).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"INSERT INTO buildings(player_id, building_type, tile_id, hp, max_hp, construction_done_tick)
                   VALUES($1,$2,$3,$4,$5,$6)"#,
                player_id, req.building_type, tile_id,
                def.base_hp, def.base_hp, done_tick
            ).execute(pool).await?;
        }
    }
    Ok(())
}
