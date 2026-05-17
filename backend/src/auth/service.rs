use crate::errors::{AppError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub:      i64,
    pub username: String,
    pub exp:      i64,
    pub iat:      i64,
}

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, 12).map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash).map_err(|e| AppError::Internal(e.into()))
}

pub fn create_token(player_id: i64, username: &str, secret: &str) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub:      player_id,
        username: username.to_string(),
        exp:      (now + Duration::days(7)).timestamp(),
        iat:      now.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))
}
