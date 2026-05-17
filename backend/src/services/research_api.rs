use crate::errors::{AppError, Result};
use crate::state::{AppState, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TechDto {
    pub tech_id:            String,
    pub name:               String,
    pub description:        String,
    pub current_level:      i64,
    pub max_level:          i64,
    pub research_done_tick: Option<i64>,
    pub eta_ticks:          Option<i64>,
    pub cost_per_level:     TechCost,
    pub ticks_per_level:    i64,
}

#[derive(Serialize, Clone)]
pub struct TechCost {
    pub wood:  f64,
    pub stone: f64,
    pub food:  f64,
    pub iron:  f64,
}

#[derive(Deserialize)]
pub struct StartResearchRequest {
    pub tech_id: String,
}

pub struct TechDef {
    pub id:              &'static str,
    pub name:            &'static str,
    pub description:     &'static str,
    pub max_level:       i64,
    pub ticks_per_level: i64,
    pub cost:            TechCost,
}

pub const TECHS: &[TechDef] = &[
    TechDef {
        id: "agriculture", name: "Agriculture", max_level: 5, ticks_per_level: 10,
        description: "Increases food production per farm level",
        cost: TechCost { wood: 50.0, stone: 20.0, food: 100.0, iron: 0.0 },
    },
    TechDef {
        id: "masonry", name: "Masonry", max_level: 5, ticks_per_level: 15,
        description: "Increases building HP and stone production",
        cost: TechCost { wood: 30.0, stone: 100.0, food: 20.0, iron: 10.0 },
    },
    TechDef {
        id: "metalworking", name: "Metalworking", max_level: 5, ticks_per_level: 20,
        description: "Unlocks iron-based units and buildings",
        cost: TechCost { wood: 40.0, stone: 40.0, food: 30.0, iron: 80.0 },
    },
    TechDef {
        id: "rocketry", name: "Rocketry", max_level: 3, ticks_per_level: 50,
        description: "Required for launch pads and orbital travel",
        cost: TechCost { wood: 0.0, stone: 200.0, food: 50.0, iron: 300.0 },
    },
    TechDef {
        id: "warp_drive", name: "Warp Drive", max_level: 3, ticks_per_level: 100,
        description: "Reduces space_travel tick cost by 30% per level",
        cost: TechCost { wood: 0.0, stone: 500.0, food: 100.0, iron: 800.0 },
    },
    TechDef {
        id: "cloaking", name: "Cloaking", max_level: 3, ticks_per_level: 60,
        description: "Allows ships to cloak in space",
        cost: TechCost { wood: 0.0, stone: 300.0, food: 0.0, iron: 400.0 },
    },
    TechDef {
        id: "sensors", name: "Sensors", max_level: 5, ticks_per_level: 30,
        description: "Increases scan range for detecting cloaked ships",
        cost: TechCost { wood: 20.0, stone: 150.0, food: 0.0, iron: 200.0 },
    },
    TechDef {
        id: "military", name: "Military Training", max_level: 10, ticks_per_level: 8,
        description: "Increases unit attack and defense by 5% per level",
        cost: TechCost { wood: 30.0, stone: 30.0, food: 80.0, iron: 50.0 },
    },
];

pub fn get_tech(id: &str) -> Option<&'static TechDef> {
    TECHS.iter().find(|t| t.id == id)
}

pub async fn list_research(state: &AppState, player_id: i64) -> Result<Vec<TechDto>> {
    let rows = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query!(
            "SELECT tech_id, level, research_done_tick FROM player_research WHERE player_id=?",
            player_id
        ).fetch_all(pool).await?.into_iter()
         .map(|r| (r.tech_id, r.level, r.research_done_tick))
         .collect::<Vec<_>>(),
        DbPool::Postgres(pool) => sqlx::query!(
            "SELECT tech_id, level, research_done_tick FROM player_research WHERE player_id=$1",
            player_id
        ).fetch_all(pool).await?.into_iter()
         .map(|r| (r.tech_id, r.level, r.research_done_tick))
         .collect::<Vec<_>>(),
    };

    let current_tick = state.current_tick() as i64;

    let mut result = Vec::new();
    for def in TECHS {
        let row = rows.iter().find(|(tid, _, _)| tid == def.id);
        let (level, done_tick) = row.map(|(_, l, d)| (*l, *d)).unwrap_or((0, None));
        let eta = done_tick.map(|t| (t - current_tick).max(0));
        result.push(TechDto {
            tech_id:            def.id.to_string(),
            name:               def.name.to_string(),
            description:        def.description.to_string(),
            current_level:      level,
            max_level:          def.max_level,
            research_done_tick: done_tick,
            eta_ticks:          eta,
            cost_per_level:     def.cost.clone(),
            ticks_per_level:    def.ticks_per_level,
        });
    }
    Ok(result)
}

pub async fn start_research(state: &AppState, player_id: i64, tech_id: &str) -> Result<TechDto> {
    let def = get_tech(tech_id)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown tech: {tech_id}")))?;

    // Check no research already in progress
    let in_progress = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT COUNT(*) FROM player_research WHERE player_id=? AND research_done_tick IS NOT NULL",
            player_id
        ).fetch_one(p).await? > 0,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT COUNT(*) FROM player_research WHERE player_id=$1 AND research_done_tick IS NOT NULL",
            player_id
        ).fetch_one(p).await?.unwrap_or(0) > 0,
    };
    if in_progress {
        return Err(AppError::Conflict("Already researching something".into()));
    }

    // Get current level
    let cur_level: i64 = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=? AND tech_id=?", player_id, tech_id
        ).fetch_optional(p).await?.unwrap_or(0),
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=$1 AND tech_id=$2", player_id, tech_id
        ).fetch_optional(p).await?.flatten().unwrap_or(0),
    };

    if cur_level >= def.max_level {
        return Err(AppError::BadRequest("Already at max level".into()));
    }

    let tick = state.current_tick() as i64;
    let done_tick = tick + def.ticks_per_level;

    match &state.db {
        DbPool::Sqlite(p) => {
            sqlx::query!(
                r#"INSERT INTO player_research(player_id,tech_id,level,research_done_tick)
                   VALUES(?,?,?,?)
                   ON CONFLICT(player_id,tech_id) DO UPDATE
                   SET research_done_tick=excluded.research_done_tick"#,
                player_id, tech_id, cur_level, done_tick
            ).execute(p).await?;
        }
        DbPool::Postgres(p) => {
            sqlx::query!(
                r#"INSERT INTO player_research(player_id,tech_id,level,research_done_tick)
                   VALUES($1,$2,$3,$4)
                   ON CONFLICT(player_id,tech_id) DO UPDATE
                   SET research_done_tick=EXCLUDED.research_done_tick"#,
                player_id, tech_id, cur_level, done_tick
            ).execute(p).await?;
        }
    }

    Ok(TechDto {
        tech_id:            def.id.to_string(),
        name:               def.name.to_string(),
        description:        def.description.to_string(),
        current_level:      cur_level,
        max_level:          def.max_level,
        research_done_tick: Some(done_tick),
        eta_ticks:          Some(def.ticks_per_level),
        cost_per_level:     def.cost.clone(),
        ticks_per_level:    def.ticks_per_level,
    })
}
