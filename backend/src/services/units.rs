/// Unit recruitment — build units at barracks/space_dock tiles.
use crate::errors::{AppError, Result};
use crate::state::{AppState, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RecruitRequest {
    pub unit_type:  String,
    pub count:      i64,
    pub planet_id:  i64,
    pub face:       i64,
    pub u:          i64,
    pub v:          i64,
}

#[derive(Serialize)]
pub struct RecruitResponse {
    pub unit_id:      i64,
    pub unit_type:    String,
    pub count:        i64,
}

pub struct UnitDef {
    pub id:         &'static str,
    pub base_hp:    i64,
    pub requires:   &'static str,   // building type needed
    pub wood_cost:  f64,
    pub stone_cost: f64,
    pub food_cost:  f64,
    pub iron_cost:  f64,
}

pub const UNIT_DEFS: &[UnitDef] = &[
    UnitDef { id:"soldier",    base_hp:50,  requires:"barracks",   wood_cost:10.0,  stone_cost:5.0,   food_cost:20.0,  iron_cost:0.0   },
    UnitDef { id:"archer",     base_hp:35,  requires:"barracks",   wood_cost:20.0,  stone_cost:5.0,   food_cost:15.0,  iron_cost:0.0   },
    UnitDef { id:"cavalry",    base_hp:80,  requires:"barracks",   wood_cost:15.0,  stone_cost:10.0,  food_cost:30.0,  iron_cost:10.0  },
    UnitDef { id:"catapult",   base_hp:60,  requires:"barracks",   wood_cost:50.0,  stone_cost:30.0,  food_cost:20.0,  iron_cost:20.0  },
    UnitDef { id:"fighter",    base_hp:100, requires:"space_dock",  wood_cost:0.0,   stone_cost:100.0, food_cost:50.0,  iron_cost:150.0 },
    UnitDef { id:"bomber",     base_hp:80,  requires:"space_dock",  wood_cost:0.0,   stone_cost:150.0, food_cost:50.0,  iron_cost:200.0 },
    UnitDef { id:"battleship", base_hp:200, requires:"space_dock",  wood_cost:0.0,   stone_cost:300.0, food_cost:100.0, iron_cost:500.0 },
    UnitDef { id:"transport",  base_hp:150, requires:"space_dock",  wood_cost:0.0,   stone_cost:200.0, food_cost:80.0,  iron_cost:300.0 },
];

pub fn get_unit_def(id: &str) -> Option<&'static UnitDef> {
    UNIT_DEFS.iter().find(|u| u.id == id)
}

pub async fn recruit(state: &AppState, player_id: i64, req: RecruitRequest) -> Result<RecruitResponse> {
    let def = get_unit_def(&req.unit_type)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown unit type: {}", req.unit_type)))?;

    if req.count < 1 || req.count > 1000 {
        return Err(AppError::BadRequest("Count must be 1–1000".into()));
    }

    // Check required building exists on a tile the player owns on the same planet
    let has_building = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM buildings b
               JOIN planet_tiles t ON t.id = b.tile_id
               WHERE b.player_id=? AND b.building_type=?
               AND t.planet_id=? AND b.construction_done_tick IS NULL
               AND b.destroyed_at IS NULL"#,
            player_id, def.requires, req.planet_id
        ).fetch_one(p).await? > 0,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM buildings b
               JOIN planet_tiles t ON t.id = b.tile_id
               WHERE b.player_id=$1 AND b.building_type=$2
               AND t.planet_id=$3 AND b.construction_done_tick IS NULL
               AND b.destroyed_at IS NULL"#,
            player_id, def.requires, req.planet_id
        ).fetch_one(p).await?.unwrap_or(0) > 0,
    };
    if !has_building {
        return Err(AppError::Forbidden(
            format!("Requires a completed {} on this planet", def.requires)
        ));
    }

    // Deduct resources
    let total = req.count as f64;
    deduct_resource(state, player_id, "wood",  def.wood_cost  * total).await?;
    deduct_resource(state, player_id, "stone", def.stone_cost * total).await?;
    deduct_resource(state, player_id, "food",  def.food_cost  * total).await?;
    deduct_resource(state, player_id, "iron",  def.iron_cost  * total).await?;

    let hp = def.base_hp * req.count;
    let unit_id = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            r#"INSERT INTO units(unit_type,is_squad,count,hp,max_hp,player_id,
                                 location_mode,planet_id,planet_face,planet_u,planet_v)
               VALUES(?,1,?,?,?,?,'planet_surface',?,?,?,?) RETURNING id"#,
            req.unit_type, req.count, hp, def.base_hp, player_id,
            req.planet_id, req.face, req.u, req.v
        ).fetch_one(p).await?,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            r#"INSERT INTO units(unit_type,is_squad,count,hp,max_hp,player_id,
                                 location_mode,planet_id,planet_face,planet_u,planet_v)
               VALUES($1,true,$2,$3,$4,$5,'planet_surface',$6,$7,$8,$9) RETURNING id"#,
            req.unit_type, req.count, hp, def.base_hp, player_id,
            req.planet_id, req.face, req.u, req.v
        ).fetch_one(p).await?,
    };

    Ok(RecruitResponse { unit_id, unit_type: req.unit_type, count: req.count })
}

async fn deduct_resource(state: &AppState, player_id: i64, rt: &str, amount: f64) -> Result<()> {
    if amount <= 0.0 { return Ok(()); }
    let affected = match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            "UPDATE player_resources SET amount = amount - ? WHERE player_id=? AND resource_type=? AND amount >= ?",
            amount, player_id, rt, amount
        ).execute(p).await?.rows_affected(),
        DbPool::Postgres(p) => sqlx::query!(
            "UPDATE player_resources SET amount = amount - $1 WHERE player_id=$2 AND resource_type=$3 AND amount >= $1",
            amount, player_id, rt
        ).execute(p).await?.rows_affected(),
    };
    if affected == 0 {
        return Err(AppError::BadRequest(format!("Not enough {rt}")));
    }
    Ok(())
}
