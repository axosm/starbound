use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SetSpeedRequest {
    pub speed: u64,
}

#[derive(Serialize)]
pub struct SpeedResponse {
    pub speed:                    u64,
    pub real_tick_ms:             u64,
    pub game_tick_ms:             u64,
    pub allow_player_speed_change: bool,
    pub current_tick:             u64,
}
