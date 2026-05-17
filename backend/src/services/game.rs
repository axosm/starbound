use crate::dto::game::{GameInitResponse, PlanetSummaryDto};
use crate::errors::Result;
use crate::procgen;
use crate::state::{AppState, DbPool};
use crate::errors::AppError;

const STARTER_RESOURCES: &[(&str, f64)] = &[
    ("wood", 500.0), ("stone", 500.0), ("food", 200.0), ("water", 200.0),
];

pub async fn init_player(state: &AppState, player_id: i64, username: &str) -> Result<GameInitResponse> {
    // Find or create home planet
    let home = get_or_create_home_planet(state, player_id).await?;

    Ok(GameInitResponse {
        player_id,
        username: username.to_string(),
        tick: state.current_tick(),
        speed: state.game_speed(),
        game_tick_ms: state.cfg.tick_ms,
        real_tick_ms: state.real_tick_ms(),
        home_planet: Some(home),
    })
}

async fn get_or_create_home_planet(state: &AppState, player_id: i64) -> Result<PlanetSummaryDto> {
    // Check if player already has a home planet (owns a tile there)
    let existing = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!(
                r#"SELECT p.id, p.star_system_id, p.seed, p.x, p.y, p.subdivision, p.planet_type
                   FROM planets p
                   JOIN planet_tiles t ON t.planet_id = p.id
                   WHERE t.owner_player_id = ?
                   LIMIT 1"#,
                player_id
            ).fetch_optional(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query!(
                r#"SELECT p.id, p.star_system_id, p.seed, p.x, p.y, p.subdivision, p.planet_type
                   FROM planets p
                   JOIN planet_tiles t ON t.planet_id = p.id
                   WHERE t.owner_player_id = $1
                   LIMIT 1"#,
                player_id
            ).fetch_optional(pool).await?
        }
    };

    if let Some(row) = existing {
        return Ok(PlanetSummaryDto {
            id: row.id,
            star_system_id: row.star_system_id,
            seed: row.seed,
            x: row.x,
            y: row.y,
            subdivision: row.subdivision as i64,
            planet_type: row.planet_type,
        });
    }

    // Generate a new home system + planet for this player
    create_starter_planet(state, player_id).await
}

async fn create_starter_planet(state: &AppState, player_id: i64) -> Result<PlanetSummaryDto> {
    // Use player_id as seed offset so every player gets a unique system
    let galaxy_seed = player_id as u64 * 9_999_991 + 1;
    let system_seed = player_id as u64 * 7_777_777 + 3;
    let planet_seed = player_id as u64 * 5_555_551 + 7;

    let (gx, gy, gz) = (player_id as f64 * 100.0, 0.0_f64, 0.0_f64);
    let (sx, sy, sz) = (10.0_f64, 0.0_f64, 0.0_f64);

    // Upsert galaxy
    let galaxy_id = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO galaxies(seed,x,y,z) VALUES(?,?,?,?) ON CONFLICT(x,y,z) DO UPDATE SET seed=excluded.seed RETURNING id",
                galaxy_seed as i64, gx, gy, gz
            ).fetch_one(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO galaxies(seed,pos) VALUES($1, ST_MakePoint($2,$3,$4)::geometry)
                 ON CONFLICT DO NOTHING RETURNING id",
                galaxy_seed as i64, gx, gy, gz
            ).fetch_one(pool).await?
        }
    };

    // Upsert star system
    let system_id = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO star_systems(galaxy_id,seed,x,y,z) VALUES(?,?,?,?,?) ON CONFLICT(galaxy_id,x,y,z) DO UPDATE SET seed=excluded.seed RETURNING id",
                galaxy_id, system_seed as i64, sx, sy, sz
            ).fetch_one(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO star_systems(galaxy_id,seed,pos) VALUES($1,$2,ST_MakePoint($3,$4,$5)::geometry)
                 ON CONFLICT DO NOTHING RETURNING id",
                galaxy_id, system_seed as i64, sx, sy, sz
            ).fetch_one(pool).await?
        }
    };

    // Create planet
    let subdivision = 4i64;
    let planet_x = 50.0_f64;
    let planet_y = 0.0_f64;
    let planet_id = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO planets(star_system_id,seed,orbit_index,x,y,subdivision,planet_type) VALUES(?,?,0,?,?,?,'terrestrial') RETURNING id",
                system_id, planet_seed as i64, planet_x, planet_y, subdivision
            ).fetch_one(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar!(
                "INSERT INTO planets(star_system_id,seed,orbit_index,pos,subdivision,planet_type) VALUES($1,$2,0,ST_MakePoint($3,$4)::geometry,$5,'terrestrial') RETURNING id",
                system_id, planet_seed as i64, planet_x, planet_y, subdivision
            ).fetch_one(pool).await?
        }
    };

    // Generate starter tiles (5×5 around origin)
    let tile_types = ["plains","forest","mountain","plains","plains"];
    let mut rng = procgen::seeded_rng(planet_seed);
    for face in 0..1i64 {
        for u in 0..5i64 {
            for v in 0..5i64 {
                let tt = tile_types[((u + v) as usize) % tile_types.len()];
                let yq = procgen::rand_f64(&mut rng, 0.3, 1.0);
                let is_home_tile = face == 0 && u == 2 && v == 2;
                let owner = if is_home_tile { Some(player_id) } else { None };
                match &state.db {
                    DbPool::Sqlite(pool) => {
                        sqlx::query!(
                            "INSERT OR IGNORE INTO planet_tiles(planet_id,face,u,v,tile_type,yield_quality,owner_player_id) VALUES(?,?,?,?,?,?,?)",
                            planet_id, face, u, v, tt, yq, owner
                        ).execute(pool).await?;
                    }
                    DbPool::Postgres(pool) => {
                        sqlx::query!(
                            "INSERT INTO planet_tiles(planet_id,face,u,v,tile_type,yield_quality,owner_player_id) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT DO NOTHING",
                            planet_id, face, u, v, tt, yq, owner
                        ).execute(pool).await?;
                    }
                }
            }
        }
    }

    Ok(PlanetSummaryDto {
        id: planet_id,
        star_system_id: system_id,
        seed: planet_seed as i64,
        x: planet_x,
        y: planet_y,
        subdivision,
        planet_type: "terrestrial".into(),
    })
}

pub async fn seed_player_resources(state: &AppState, player_id: i64) -> Result<()> {
    for (rt, amount) in STARTER_RESOURCES {
        match &state.db {
            DbPool::Sqlite(pool) => {
                sqlx::query!(
                    "INSERT OR IGNORE INTO player_resources(player_id,resource_type,amount,cap) VALUES(?,?,?,1000)",
                    player_id, rt, amount
                ).execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query!(
                    "INSERT INTO player_resources(player_id,resource_type,amount,cap) VALUES($1,$2,$3,1000) ON CONFLICT DO NOTHING",
                    player_id, rt, amount
                ).execute(pool).await?;
            }
        }
    }
    Ok(())
}
