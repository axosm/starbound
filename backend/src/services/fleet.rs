use crate::dto::fleet::{MoveOrderResponse, MoveOrderDto, MoveUnitRequest, UnitDto};
use crate::errors::{AppError, Result};
use crate::game::movement;
use crate::sse;
use crate::state::{AppState, DbPool};

pub async fn list_units(state: &AppState, player_id: i64) -> Result<Vec<UnitDto>> {
    let tick = state.current_tick() as i64;
    let speed = state.game_speed();
    let tick_ms = state.cfg.tick_ms;

    let units = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(r#"
                SELECT u.id, u.unit_type, u.hp, u.max_hp, u.count, u.location_mode,
                       u.planet_id, u.planet_face, u.planet_u, u.planet_v,
                       u.orbit_planet_id, u.star_system_id, u.space_x, u.space_y, u.space_z,
                       u.in_battle,
                       m.id as order_id, m.move_type, m.start_tick, m.arrival_tick
                FROM units u
                LEFT JOIN move_orders m ON m.unit_id = u.id
                WHERE u.player_id = ?
            "#, player_id).fetch_all(pool).await?
            .into_iter()
            .map(|r| {
                let move_order = r.order_id.map(|oid| {
                    let ticks_left = (r.arrival_tick.unwrap_or(0) - tick).max(0);
                    let eta_secs = ticks_left as f64 * (tick_ms as f64 / 1000.0) / speed as f64;
                    MoveOrderDto {
                        order_id: oid,
                        move_type: r.move_type.unwrap_or_default(),
                        start_tick: r.start_tick.unwrap_or(0),
                        arrival_tick: r.arrival_tick.unwrap_or(0),
                        ticks_left,
                        eta_seconds: eta_secs,
                    }
                });
                UnitDto {
                    id: r.id,
                    unit_type: r.unit_type,
                    hp: r.hp,
                    max_hp: r.max_hp,
                    count: r.count,
                    location_mode: r.location_mode,
                    planet_id: r.planet_id,
                    planet_face: r.planet_face,
                    planet_u: r.planet_u,
                    planet_v: r.planet_v,
                    orbit_planet_id: r.orbit_planet_id,
                    star_system_id: r.star_system_id,
                    space_x: r.space_x,
                    space_y: r.space_y,
                    space_z: r.space_z,
                    in_battle: r.in_battle != 0,
                    move_order,
                }
            })
            .collect()
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(r#"
                SELECT u.id, u.unit_type, u.hp, u.max_hp, u.count, u.location_mode,
                       u.planet_id, u.planet_face, u.planet_u, u.planet_v,
                       u.orbit_planet_id, u.star_system_id,
                       ST_X(u.space_pos) as space_x, ST_Y(u.space_pos) as space_y,
                       ST_Z(u.space_pos) as space_z,
                       u.in_battle,
                       m.id as order_id, m.move_type, m.start_tick, m.arrival_tick
                FROM units u
                LEFT JOIN move_orders m ON m.unit_id = u.id
                WHERE u.player_id = $1
            "#, player_id).fetch_all(pool).await?
            .into_iter()
            .map(|r| {
                let move_order = r.order_id.map(|oid| {
                    let ticks_left = (r.arrival_tick.unwrap_or(0) - tick).max(0);
                    let eta_secs = ticks_left as f64 * (tick_ms as f64 / 1000.0) / speed as f64;
                    MoveOrderDto {
                        order_id: oid,
                        move_type: r.move_type.unwrap_or_default(),
                        start_tick: r.start_tick.unwrap_or(0),
                        arrival_tick: r.arrival_tick.unwrap_or(0),
                        ticks_left,
                        eta_seconds: eta_secs,
                    }
                });
                UnitDto {
                    id: r.id,
                    unit_type: r.unit_type,
                    hp: r.hp,
                    max_hp: r.max_hp,
                    count: r.count,
                    location_mode: r.location_mode,
                    planet_id: r.planet_id,
                    planet_face: r.planet_face,
                    planet_u: r.planet_u,
                    planet_v: r.planet_v,
                    orbit_planet_id: r.orbit_planet_id,
                    star_system_id: r.star_system_id,
                    space_x: r.space_x.map(|v| v as f64),
                    space_y: r.space_y.map(|v| v as f64),
                    space_z: r.space_z.map(|v| v as f64),
                    in_battle: r.in_battle,
                    move_order,
                }
            })
            .collect()
        }
    };
    Ok(units)
}

pub async fn issue_move_order(
    state: &AppState,
    player_id: i64,
    req: MoveUnitRequest,
) -> Result<MoveOrderResponse> {
    // Validate unit ownership
    let unit_exists = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM units WHERE id=? AND player_id=?",
                req.unit_id, player_id
            ).fetch_one(pool).await? > 0
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM units WHERE id=$1 AND player_id=$2",
                req.unit_id, player_id
            ).fetch_one(pool).await?.unwrap_or(0) > 0
        }
    };
    if !unit_exists {
        return Err(AppError::Forbidden("Unit not found or not yours".into()));
    }

    // Check no existing order
    let has_order = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM move_orders WHERE unit_id=?",
                req.unit_id
            ).fetch_one(pool).await? > 0
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM move_orders WHERE unit_id=$1",
                req.unit_id
            ).fetch_one(pool).await?.unwrap_or(0) > 0
        }
    };
    if has_order {
        return Err(AppError::Conflict("Unit already has a move order in progress".into()));
    }

    let tick = state.current_tick() as i64;
    let duration_ticks = movement::travel_ticks(&req.move_type);
    let arrival_tick = tick + duration_ticks;

    let order_id = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                r#"INSERT INTO move_orders(
                    unit_id, mover_type, move_type,
                    to_planet_id, to_planet_face, to_planet_u, to_planet_v,
                    to_star_system_id, to_space_x, to_space_y, to_space_z,
                    start_tick, arrival_tick
                ) VALUES(?,?,?, ?,?,?,?, ?,?,?,?, ?,?) RETURNING id"#,
                req.unit_id, "unit", req.move_type,
                req.to_planet_id, req.to_face, req.to_u, req.to_v,
                req.to_system_id, req.to_x, req.to_y, req.to_z,
                tick, arrival_tick
            ).fetch_one(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                r#"INSERT INTO move_orders(
                    unit_id, mover_type, move_type,
                    to_planet_id, to_planet_face, to_planet_u, to_planet_v,
                    to_star_system_id, to_pos,
                    start_tick, arrival_tick
                ) VALUES($1,$2,$3, $4,$5,$6,$7, $8,
                    CASE WHEN $9::float8 IS NOT NULL THEN ST_MakePoint($9,$10,$11)::geometry ELSE NULL END,
                    $12,$13) RETURNING id"#,
                req.unit_id, "unit", req.move_type,
                req.to_planet_id, req.to_face, req.to_u, req.to_v,
                req.to_system_id, req.to_x, req.to_y, req.to_z.unwrap_or(0.0),
                tick, arrival_tick
            ).fetch_one(pool).await?
        }
    };

    let eta_seconds = duration_ticks as f64
        * (state.cfg.tick_ms as f64 / 1000.0)
        / state.game_speed() as f64;

    Ok(MoveOrderResponse {
        order_id,
        arrival_tick,
        eta_seconds,
    })
}

/// Called each tick — find orders whose arrival_tick <= current tick and apply them.
pub async fn resolve_arrivals(state: &AppState, tick: u64) -> anyhow::Result<()> {
    let tick = tick as i64;

    struct Order {
        id: i64,
        unit_id: Option<i64>,
        move_type: String,
        to_planet_id: Option<i64>,
        to_planet_face: Option<i64>,
        to_planet_u: Option<i64>,
        to_planet_v: Option<i64>,
        to_star_system_id: Option<i64>,
        to_space_x: Option<f64>,
        to_space_y: Option<f64>,
        to_space_z: Option<f64>,
    }

    let orders = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_as!(Order,
                r#"SELECT id, unit_id, move_type,
                          to_planet_id, to_planet_face, to_planet_u, to_planet_v,
                          to_star_system_id, to_space_x, to_space_y, to_space_z
                   FROM move_orders WHERE arrival_tick <= ? AND unit_id IS NOT NULL"#,
                tick
            ).fetch_all(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_as!(Order,
                r#"SELECT id, unit_id, move_type,
                          to_planet_id, to_planet_face, to_planet_u, to_planet_v,
                          to_star_system_id,
                          ST_X(to_pos::geometry) as to_space_x,
                          ST_Y(to_pos::geometry) as to_space_y,
                          ST_Z(to_pos::geometry) as to_space_z
                   FROM move_orders WHERE arrival_tick <= $1 AND unit_id IS NOT NULL"#,
                tick
            ).fetch_all(pool).await?
        }
    };

    for order in orders {
        let unit_id = match order.unit_id { Some(id) => id, None => continue };

        // Determine new location_mode and position
        let (new_mode, planet_id, face, u, v, orbit_id, sys_id, sx, sy, sz) =
            movement::resolve_destination(&order.move_type,
                order.to_planet_id, order.to_planet_face,
                order.to_planet_u, order.to_planet_v,
                order.to_star_system_id,
                order.to_space_x, order.to_space_y, order.to_space_z,
            );

        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"UPDATE units SET
                        location_mode = ?,
                        planet_id = ?, planet_face = ?, planet_u = ?, planet_v = ?,
                        orbit_planet_id = ?,
                        star_system_id = ?, space_x = ?, space_y = ?, space_z = ?
                       WHERE id = ?"#,
                    new_mode, planet_id, face, u, v, orbit_id,
                    sys_id, sx, sy, sz, unit_id
                ).execute(pool).await?;
                sqlx::query!("DELETE FROM move_orders WHERE id=?", order.id)
                    .execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    r#"UPDATE units SET
                        location_mode = $1,
                        planet_id = $2, planet_face = $3, planet_u = $4, planet_v = $5,
                        orbit_planet_id = $6,
                        star_system_id = $7,
                        space_pos = CASE WHEN $8::float8 IS NOT NULL
                            THEN ST_MakePoint($8,$9,$10)::geometry ELSE NULL END
                       WHERE id = $11"#,
                    new_mode, planet_id, face, u, v, orbit_id,
                    sys_id, sx, sy, sz.unwrap_or(0.0), unit_id
                ).execute(pool).await?;
                sqlx::query!("DELETE FROM move_orders WHERE id=$1", order.id)
                    .execute(pool).await?;
            }
        }

        // Check for battles at new location
        if new_mode == "planet_surface" {
            if let (Some(pid), Some(f), Some(uu), Some(vv)) = (planet_id, face, u, v) {
                check_and_start_battle(state, unit_id, pid, f, uu, vv).await?;
            }
        }
    }

    Ok(())
}

async fn check_and_start_battle(
    state: &AppState,
    arriving_unit_id: i64,
    planet_id: i64,
    face: i64,
    u: i64,
    v: i64,
) -> anyhow::Result<()> {
    // Get arriving unit's owner
    let owner = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            "SELECT player_id FROM units WHERE id=?", arriving_unit_id
        ).fetch_one(pool).await?,
        DbPool::Postgres(pool) => sqlx::query_scalar!(
            "SELECT player_id FROM units WHERE id=$1", arriving_unit_id
        ).fetch_one(pool).await?,
    };

    // Find enemies on the same tile
    let enemy_id: Option<i64> = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            r#"SELECT DISTINCT u.player_id FROM units u
               WHERE u.planet_id=? AND u.planet_face=? AND u.planet_u=? AND u.planet_v=?
               AND u.player_id != ? AND u.location_mode='planet_surface'
               LIMIT 1"#,
            planet_id, face, u, v, owner
        ).fetch_optional(pool).await?,
        DbPool::Postgres(pool) => sqlx::query_scalar!(
            r#"SELECT DISTINCT u.player_id FROM units u
               WHERE u.planet_id=$1 AND u.planet_face=$2 AND u.planet_u=$3 AND u.planet_v=$4
               AND u.player_id != $5 AND u.location_mode='planet_surface'
               LIMIT 1"#,
            planet_id, face, u, v, owner
        ).fetch_optional(pool).await?,
    };

    if let Some(defender_id) = enemy_id {
        // Get tile_id
        let tile_id: Option<i64> = match &state.db {
            DbPool::Sqlite(pool) => sqlx::query_scalar!(
                "SELECT id FROM planet_tiles WHERE planet_id=? AND face=? AND u=? AND v=?",
                planet_id, face, u, v
            ).fetch_optional(pool).await?,
            DbPool::Postgres(pool) => sqlx::query_scalar!(
                "SELECT id FROM planet_tiles WHERE planet_id=$1 AND face=$2 AND u=$3 AND v=$4",
                planet_id, face, u, v
            ).fetch_optional(pool).await?,
        };

        // Insert battle if none exists for this tile
        let current_tick = state.current_tick() as i64;
        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    r#"INSERT OR IGNORE INTO battles(tile_id, attacker_id, defender_id, started_tick, last_tick)
                       VALUES(?,?,?,?,?)"#,
                    tile_id, owner, defender_id, current_tick, current_tick
                ).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    r#"INSERT INTO battles(tile_id, attacker_id, defender_id, started_tick, last_tick)
                       VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING"#,
                    tile_id, owner, defender_id, current_tick, current_tick
                ).execute(pool).await?;
            }
        }

        // Alert both players via SSE
        let payload = serde_json::json!({
            "planet_id": planet_id, "face": face, "u": u, "v": v,
            "attacker_id": owner, "defender_id": defender_id,
        });
        sse::persist_event(state, Some(defender_id), "battle_started", &payload).await.ok();
        sse::persist_event(state, Some(owner), "battle_started", &payload).await.ok();
        state.sse.broadcast("battle_started", payload);
    }
    Ok(())
}
