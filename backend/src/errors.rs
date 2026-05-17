use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]        NotFound(String),
    #[error("Unauthorized: {0}")]     Unauthorized(String),
    #[error("Forbidden: {0}")]        Forbidden(String),
    #[error("Bad request: {0}")]      BadRequest(String),
    #[error("Conflict: {0}")]         Conflict(String),
    #[error("Database error: {0}")]   Db(#[from] sqlx::Error),
    #[error("Internal: {0}")]         Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound(m)     => (StatusCode::NOT_FOUND,            m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED,          m.clone()),
            AppError::Forbidden(m)    => (StatusCode::FORBIDDEN,             m.clone()),
            AppError::BadRequest(m)   => (StatusCode::BAD_REQUEST,           m.clone()),
            AppError::Conflict(m)     => (StatusCode::CONFLICT,              m.clone()),
            AppError::Db(e)           => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Internal(e)     => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        tracing::error!("AppError [{status}]: {msg}");
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
