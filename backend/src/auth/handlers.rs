use crate::auth::service::{create_token, hash_password, verify_password};
use crate::dto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use crate::errors::{AppError, Result};
use crate::state::{AppState, DbPool};
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>> {
    if req.username.len() < 3 {
        return Err(AppError::BadRequest("Username too short (min 3)".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password too short (min 8)".into()));
    }
    let hash = hash_password(&req.password)?;

    let id = match &state.db {
        DbPool::Sqlite(pool) => sqlx::query_scalar!(
            "INSERT INTO players(username,email,password_hash) VALUES(?,?,?) RETURNING id",
            req.username, req.email, hash
        ).fetch_one(pool).await
         .map_err(|e| if e.to_string().contains("UNIQUE") {
             AppError::Conflict("Email already registered".into())
         } else { AppError::Db(e) })?,

        DbPool::Postgres(pool) => sqlx::query_scalar!(
            "INSERT INTO players(username,email,password_hash) VALUES($1,$2,$3) RETURNING id",
            req.username, req.email, hash
        ).fetch_one(pool).await
         .map_err(|e| if e.to_string().contains("unique") {
             AppError::Conflict("Email already registered".into())
         } else { AppError::Db(e) })?,
    };

    // Seed starter resources
    crate::services::game::seed_player_resources(&state, id).await?;

    let token = create_token(id, &req.username, &state.cfg.jwt_secret)?;
    Ok(Json(LoginResponse { token, player_id: id, username: req.username }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    struct Row { id: i64, username: String, password_hash: String }

    let row = match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query_as!(Row,
                "SELECT id, username, password_hash FROM players WHERE email = ?",
                req.email
            ).fetch_optional(pool).await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_as!(Row,
                "SELECT id, username, password_hash FROM players WHERE email = $1",
                req.email
            ).fetch_optional(pool).await?
        }
    };

    let row = row.ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;
    if !verify_password(&req.password, &row.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    // Update last_login_at
    match &state.db {
        DbPool::Sqlite(pool) => {
            sqlx::query!("UPDATE players SET last_login_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?", row.id)
                .execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query!("UPDATE players SET last_login_at = now() WHERE id = $1", row.id)
                .execute(pool).await?;
        }
    }

    let token = create_token(row.id, &row.username, &state.cfg.jwt_secret)?;
    Ok(Json(LoginResponse { token, player_id: row.id, username: row.username }))
}
