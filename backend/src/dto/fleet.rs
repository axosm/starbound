use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MoveUnitRequest {
    pub unit_id:      i64,
    pub move_type:    String,
    pub to_planet_id: Option<i64>,
    pub to_face:      Option<i64>,
    pub to_u:         Option<i64>,
    pub to_v:         Option<i64>,
    pub to_system_id: Option<i64>,
    pub to_x:         Option<f64>,
    pub to_y:         Option<f64>,
    pub to_z:         Option<f64>,
}

#[derive(Serialize)]
pub struct MoveOrderResponse {
    pub order_id:     i64,
    pub arrival_tick: i64,
    /// Estimated real seconds until arrival at current speed
    pub eta_seconds:  f64,
}

#[derive(Serialize)]
pub struct UnitDto {
    pub id:            i64,
    pub unit_type:     String,
    pub hp:            i64,
    pub max_hp:        i64,
    pub count:         i64,
    pub location_mode: String,
    pub planet_id:     Option<i64>,
    pub planet_face:   Option<i64>,
    pub planet_u:      Option<i64>,
    pub planet_v:      Option<i64>,
    pub orbit_planet_id: Option<i64>,
    pub star_system_id:  Option<i64>,
    pub space_x:         Option<f64>,
    pub space_y:         Option<f64>,
    pub space_z:         Option<f64>,
    pub in_battle:       bool,
    pub move_order:      Option<MoveOrderDto>,
}

#[derive(Serialize)]
pub struct MoveOrderDto {
    pub order_id:     i64,
    pub move_type:    String,
    pub start_tick:   i64,
    pub arrival_tick: i64,
    /// Ticks remaining at current tick
    pub ticks_left:   i64,
    /// Real seconds remaining at current speed
    pub eta_seconds:  f64,
}
