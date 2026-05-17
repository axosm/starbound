/// Space operations: scan, cloak, orbit management, battle reports.
use crate::errors::{AppError, Result};
use crate::state::{AppState, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ScanResult {
    pub unit_id:   i64,
    pub unit_type: String,
    pub player_id: i64,
    pub cloaked:   bool,
    pub distance:  f64,
    pub x: f64, pub y: f64, pub z: f64,
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub scanner_unit_id: i64,
    pub radius:          f64,
}

#[derive(Deserialize)]
pub struct CloakRequest {
    pub unit_id: i64,
    pub cloak:   bool,
}

#[derive(Serialize)]
pub struct BattleReportDto {
    pub id:                      i64,
    pub battle_id:               i64,
    pub attacker_id:             i64,
    pub defender_id:             i64,
    pub outcome:                 String,
    pub attacker_units_snapshot: serde_json::Value,
    pub defender_units_snapshot: serde_json::Value,
    pub resources_looted:        Option<serde_json::Value>,
    pub started_tick:            i64,
    pub ended_tick:              i64,
    pub read:                    bool,
}

/// Scan nearby space for units within radius (SQLite: manual distance calc).
pub async fn scan(state: &AppState, player_id: i64, req: ScanRequest) -> Result<Vec<ScanResult>> {
    // Get scanner position
    let (sx, sy, sz) = match &state.db {
        DbPool::Sqlite(p) => {
            let r = sqlx::query!(
                "SELECT space_x, space_y, space_z FROM units WHERE id=? AND player_id=?",
                req.scanner_unit_id, player_id
            ).fetch_optional(p).await?
             .ok_or_else(|| AppError::NotFound("Scanner unit not found".into()))?;
            (r.space_x.unwrap_or(0.0), r.space_y.unwrap_or(0.0), r.space_z.unwrap_or(0.0))
        }
        DbPool::Postgres(p) => {
            let r = sqlx::query!(
                "SELECT ST_X(space_pos) as x, ST_Y(space_pos) as y, ST_Z(space_pos) as z
                 FROM units WHERE id=$1 AND player_id=$2",
                req.scanner_unit_id, player_id
            ).fetch_optional(p).await?
             .ok_or_else(|| AppError::NotFound("Scanner unit not found".into()))?;
            (r.x.unwrap_or(0.0) as f64, r.y.unwrap_or(0.0) as f64, r.z.unwrap_or(0.0) as f64)
        }
    };

    // Get player's sensor level for scan range bonus
    let sensor_level: i64 = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=? AND tech_id='sensors'", player_id
        ).fetch_optional(p).await?.unwrap_or(0),
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=$1 AND tech_id='sensors'", player_id
        ).fetch_optional(p).await?.flatten().unwrap_or(0),
    };
    let effective_radius = req.radius * (1.0 + sensor_level as f64 * 0.2);

    // SQLite: pull all space units and filter in Rust (no PostGIS)
    // Postgres: use ST_DWithin
    let nearby = match &state.db {
        DbPool::Sqlite(p) => {
            let all = sqlx::query!(
                r#"SELECT id, unit_type, player_id, cloaked, space_x, space_y, space_z
                   FROM units WHERE location_mode='in_space' AND id != ?"#,
                req.scanner_unit_id
            ).fetch_all(p).await?;
            all.into_iter().filter_map(|r| {
                let (ux, uy, uz) = (r.space_x?, r.space_y?, r.space_z?);
                let dist = ((ux-sx).powi(2)+(uy-sy).powi(2)+(uz-sz).powi(2)).sqrt();
                if dist > effective_radius { return None; }
                let cloaked = r.cloaked != 0;
                if cloaked && r.player_id != player_id { return None; }
                Some(ScanResult { unit_id: r.id, unit_type: r.unit_type, player_id: r.player_id,
                    cloaked, distance: dist, x: ux, y: uy, z: uz })
            }).collect()
        }
        DbPool::Postgres(p) => {
            sqlx::query!(
                r#"SELECT id, unit_type, player_id, cloaked,
                          ST_X(space_pos) as x, ST_Y(space_pos) as y, ST_Z(space_pos) as z,
                          ST_Distance(space_pos, ST_MakePoint($2,$3,$4)::geometry) as dist
                   FROM units
                   WHERE location_mode='in_space'
                   AND id != $1
                   AND ST_DWithin(space_pos, ST_MakePoint($2,$3,$4)::geometry, $5)
                   AND (NOT cloaked OR player_id = $6)"#,
                req.scanner_unit_id, sx, sy, sz, effective_radius, player_id
            ).fetch_all(p).await?.into_iter().map(|r| ScanResult {
                unit_id:   r.id,
                unit_type: r.unit_type,
                player_id: r.player_id,
                cloaked:   r.cloaked,
                distance:  r.dist.unwrap_or(0.0) as f64,
                x: r.x.unwrap_or(0.0) as f64,
                y: r.y.unwrap_or(0.0) as f64,
                z: r.z.unwrap_or(0.0) as f64,
            }).collect()
        }
    };
    Ok(nearby)
}

/// Toggle cloak on a space unit (requires cloaking tech level >= 1).
pub async fn set_cloak(state: &AppState, player_id: i64, req: CloakRequest) -> Result<()> {
    let has_tech = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=? AND tech_id='cloaking'", player_id
        ).fetch_optional(p).await?.unwrap_or(0) >= 1,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT level FROM player_research WHERE player_id=$1 AND tech_id='cloaking'", player_id
        ).fetch_optional(p).await?.flatten().unwrap_or(0) >= 1,
    };
    if !has_tech {
        return Err(AppError::Forbidden("Requires Cloaking technology level 1".into()));
    }

    match &state.db {
        DbPool::Sqlite(p) => {
            let cloaked = if req.cloak { 1i64 } else { 0i64 };
            sqlx::query!(
                "UPDATE units SET cloaked=? WHERE id=? AND player_id=? AND location_mode='in_space'",
                cloaked, req.unit_id, player_id
            ).execute(p).await?;
        }
        DbPool::Postgres(p) => {
            sqlx::query!(
                "UPDATE units SET cloaked=$1 WHERE id=$2 AND player_id=$3 AND location_mode='in_space'",
                req.cloak, req.unit_id, player_id
            ).execute(p).await?;
        }
    }
    Ok(())
}

/// Get battle reports for a player, most recent first.
pub async fn battle_reports(state: &AppState, player_id: i64, limit: i64) -> Result<Vec<BattleReportDto>> {
    let reports = match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            r#"SELECT id, battle_id, attacker_id, defender_id, outcome,
                      attacker_units_snapshot, defender_units_snapshot,
                      resources_looted_json, started_tick, ended_tick,
                      CASE WHEN attacker_id=? THEN attacker_read ELSE defender_read END as my_read
               FROM battle_reports
               WHERE attacker_id=? OR defender_id=?
               ORDER BY ended_tick DESC LIMIT ?"#,
            player_id, player_id, player_id, limit
        ).fetch_all(p).await?.into_iter().map(|r| BattleReportDto {
            id: r.id,
            battle_id: r.battle_id,
            attacker_id: r.attacker_id,
            defender_id: r.defender_id,
            outcome: r.outcome,
            attacker_units_snapshot: serde_json::from_str(&r.attacker_units_snapshot).unwrap_or_default(),
            defender_units_snapshot: serde_json::from_str(&r.defender_units_snapshot).unwrap_or_default(),
            resources_looted: r.resources_looted_json.and_then(|s| serde_json::from_str(&s).ok()),
            started_tick: r.started_tick,
            ended_tick: r.ended_tick,
            read: r.my_read != 0,
        }).collect(),
        DbPool::Postgres(p) => sqlx::query!(
            r#"SELECT id, battle_id, attacker_id, defender_id, outcome,
                      attacker_units_snapshot::text as attacker_snap,
                      defender_units_snapshot::text as defender_snap,
                      resources_looted_json::text as looted,
                      started_tick, ended_tick,
                      CASE WHEN attacker_id=$1 THEN attacker_read ELSE defender_read END as my_read
               FROM battle_reports
               WHERE attacker_id=$1 OR defender_id=$1
               ORDER BY ended_tick DESC LIMIT $2"#,
            player_id, limit
        ).fetch_all(p).await?.into_iter().map(|r| BattleReportDto {
            id: r.id,
            battle_id: r.battle_id,
            attacker_id: r.attacker_id,
            defender_id: r.defender_id,
            outcome: r.outcome,
            attacker_units_snapshot: r.attacker_snap.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            defender_units_snapshot: r.defender_snap.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            resources_looted: r.looted.and_then(|s| serde_json::from_str(&s).ok()),
            started_tick: r.started_tick,
            ended_tick: r.ended_tick,
            read: r.my_read.unwrap_or(false),
        }).collect(),
    };
    Ok(reports)
}

/// Mark battle reports as read.
pub async fn mark_reports_read(state: &AppState, player_id: i64) -> Result<()> {
    match &state.db {
        DbPool::Sqlite(p) => {
            sqlx::query!("UPDATE battle_reports SET attacker_read=1 WHERE attacker_id=?", player_id)
                .execute(p).await?;
            sqlx::query!("UPDATE battle_reports SET defender_read=1 WHERE defender_id=?", player_id)
                .execute(p).await?;
        }
        DbPool::Postgres(p) => {
            sqlx::query!("UPDATE battle_reports SET attacker_read=true WHERE attacker_id=$1", player_id)
                .execute(p).await?;
            sqlx::query!("UPDATE battle_reports SET defender_read=true WHERE defender_id=$1", player_id)
                .execute(p).await?;
        }
    }
    Ok(())
}
