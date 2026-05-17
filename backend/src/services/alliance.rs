use crate::errors::{AppError, Result};
use crate::state::{AppState, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct EmpireDto {
    pub id:         i64,
    pub name:       String,
    pub created_by: i64,
    pub members:    Vec<MemberDto>,
}

#[derive(Serialize)]
pub struct MemberDto {
    pub player_id: i64,
    pub username:  String,
    pub role:      String,
}

#[derive(Deserialize)]
pub struct CreateEmpireRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct InviteRequest {
    pub target_username: String,
}

#[derive(Deserialize)]
pub struct SetRoleRequest {
    pub target_player_id: i64,
    pub role:             String,
}

pub async fn create_empire(state: &AppState, player_id: i64, name: &str) -> Result<EmpireDto> {
    if name.trim().len() < 2 {
        return Err(AppError::BadRequest("Empire name too short".into()));
    }
    let in_empire = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            "SELECT COUNT(*) FROM empire_members WHERE player_id = ?", player_id
        ).fetch_one(pool).await? > 0,
        DbPool::Postgres(pool) => sqlx::query_scalar!(
            "SELECT COUNT(*) FROM empire_members WHERE player_id = $1", player_id
        ).fetch_one(pool).await?.unwrap_or(0) > 0,
    };
    if in_empire {
        return Err(AppError::Conflict("Already in an empire".into()));
    }

    let empire_id = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            "INSERT INTO empires(name, created_by) VALUES(?,?) RETURNING id",
            name, player_id
        ).fetch_one(pool).await.map_err(|e| {
            if e.to_string().contains("UNIQUE") { AppError::Conflict("Name taken".into()) }
            else { AppError::Db(e) }
        })?,
        DbPool::Postgres(pool) => sqlx::query_scalar!(
            "INSERT INTO empires(name, created_by) VALUES($1,$2) RETURNING id",
            name, player_id
        ).fetch_one(pool).await.map_err(|e| {
            if e.to_string().contains("unique") { AppError::Conflict("Name taken".into()) }
            else { AppError::Db(e) }
        })?,
    };

    match &state.db {
        DbPool::Sqlite(pool) => sqlx::query!(
            "INSERT INTO empire_members(empire_id,player_id,role) VALUES(?,?,'leader')",
            empire_id, player_id
        ).execute(pool).await?,
        DbPool::Postgres(pool) => sqlx::query!(
            "INSERT INTO empire_members(empire_id,player_id,role) VALUES($1,$2,'leader')",
            empire_id, player_id
        ).execute(pool).await?,
    };

    get_empire(state, empire_id).await
}

pub async fn get_empire(state: &AppState, empire_id: i64) -> Result<EmpireDto> {
    let (name, created_by) = match &state.db {
        DbPool::Sqlite(pool) => {
            let r = sqlx::query!("SELECT name, created_by FROM empires WHERE id=?", empire_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::NotFound("Empire not found".into()))?;
            (r.name, r.created_by)
        }
        DbPool::Postgres(pool) => {
            let r = sqlx::query!("SELECT name, created_by FROM empires WHERE id=$1", empire_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::NotFound("Empire not found".into()))?;
            (r.name, r.created_by)
        }
    };

    let members = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query!(
            "SELECT em.player_id, p.username, em.role FROM empire_members em
             JOIN players p ON p.id=em.player_id WHERE em.empire_id=?", empire_id
        ).fetch_all(pool).await?.into_iter()
         .map(|r| MemberDto { player_id: r.player_id, username: r.username, role: r.role })
         .collect(),
        DbPool::Postgres(pool) => sqlx::query!(
            "SELECT em.player_id, p.username, em.role FROM empire_members em
             JOIN players p ON p.id=em.player_id WHERE em.empire_id=$1", empire_id
        ).fetch_all(pool).await?.into_iter()
         .map(|r| MemberDto { player_id: r.player_id, username: r.username, role: r.role })
         .collect(),
    };

    Ok(EmpireDto { id: empire_id, name, created_by, members })
}

pub async fn my_empire(state: &AppState, player_id: i64) -> Result<Option<EmpireDto>> {
    let id: Option<i64> = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT empire_id FROM empire_members WHERE player_id=?", player_id
        ).fetch_optional(p).await?,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT empire_id FROM empire_members WHERE player_id=$1", player_id
        ).fetch_optional(p).await?,
    };
    match id {
        None => Ok(None),
        Some(eid) => Ok(Some(get_empire(state, eid).await?)),
    }
}

pub async fn invite_player(state: &AppState, empire_id: i64, requester: i64, target: &str) -> Result<()> {
    require_role(state, empire_id, requester, &["leader","officer"]).await?;
    let tid = find_player(state, target).await?
        .ok_or_else(|| AppError::NotFound(format!("Player '{target}' not found")))?;
    match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            "INSERT INTO empire_members(empire_id,player_id,role) VALUES(?,?,'member')", empire_id, tid
        ).execute(p).await?,
        DbPool::Postgres(p) => sqlx::query!(
            "INSERT INTO empire_members(empire_id,player_id,role) VALUES($1,$2,'member')", empire_id, tid
        ).execute(p).await?,
    };
    Ok(())
}

pub async fn kick_player(state: &AppState, empire_id: i64, requester: i64, target: i64) -> Result<()> {
    require_role(state, empire_id, requester, &["leader"]).await?;
    if requester == target { return Err(AppError::BadRequest("Cannot kick yourself".into())); }
    match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            "DELETE FROM empire_members WHERE empire_id=? AND player_id=?", empire_id, target
        ).execute(p).await?,
        DbPool::Postgres(p) => sqlx::query!(
            "DELETE FROM empire_members WHERE empire_id=$1 AND player_id=$2", empire_id, target
        ).execute(p).await?,
    };
    Ok(())
}

pub async fn leave_empire(state: &AppState, player_id: i64) -> Result<()> {
    match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            "DELETE FROM empire_members WHERE player_id=?", player_id
        ).execute(p).await?,
        DbPool::Postgres(p) => sqlx::query!(
            "DELETE FROM empire_members WHERE player_id=$1", player_id
        ).execute(p).await?,
    };
    Ok(())
}

pub async fn set_role(state: &AppState, empire_id: i64, requester: i64, target: i64, role: &str) -> Result<()> {
    if !matches!(role, "officer"|"member") {
        return Err(AppError::BadRequest("Role must be officer or member".into()));
    }
    require_role(state, empire_id, requester, &["leader"]).await?;
    match &state.db {
        DbPool::Sqlite(p) => sqlx::query!(
            "UPDATE empire_members SET role=? WHERE empire_id=? AND player_id=?", role, empire_id, target
        ).execute(p).await?,
        DbPool::Postgres(p) => sqlx::query!(
            "UPDATE empire_members SET role=$1 WHERE empire_id=$2 AND player_id=$3", role, empire_id, target
        ).execute(p).await?,
    };
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────

async fn require_role(state: &AppState, empire_id: i64, player_id: i64, allowed: &[&str]) -> Result<()> {
    let role: Option<String> = match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT role FROM empire_members WHERE empire_id=? AND player_id=?", empire_id, player_id
        ).fetch_optional(p).await?,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT role FROM empire_members WHERE empire_id=$1 AND player_id=$2", empire_id, player_id
        ).fetch_optional(p).await?,
    };
    let role = role.ok_or_else(|| AppError::Forbidden("Not a member of this empire".into()))?;
    if !allowed.contains(&role.as_str()) {
        return Err(AppError::Forbidden(format!("Requires role: {}", allowed.join(" or "))));
    }
    Ok(())
}

async fn find_player(state: &AppState, username: &str) -> Result<Option<i64>> {
    Ok(match &state.db {
        DbPool::Sqlite(p) => sqlx::query_scalar!(
            "SELECT id FROM players WHERE username=?", username
        ).fetch_optional(p).await?,
        DbPool::Postgres(p) => sqlx::query_scalar!(
            "SELECT id FROM players WHERE username=$1", username
        ).fetch_optional(p).await?,
    })
}
