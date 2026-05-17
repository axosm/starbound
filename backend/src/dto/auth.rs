use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email:    String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token:     String,
    pub player_id: i64,
    pub username:  String,
}
