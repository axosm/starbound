use serde::Serialize;

#[derive(Serialize)]
pub struct GameInitResponse {
    pub player_id:    i64,
    pub username:     String,
    pub tick:         u64,
    pub speed:        u64,
    pub game_tick_ms: u64,
    pub real_tick_ms: u64,
    pub home_planet:  Option<PlanetSummaryDto>,
}

#[derive(Serialize, Clone)]
pub struct PlanetSummaryDto {
    pub id:             i64,
    pub star_system_id: i64,
    pub seed:           i64,
    pub x:              f64,
    pub y:              f64,
    pub subdivision:    i64,
    pub planet_type:    String,
}
