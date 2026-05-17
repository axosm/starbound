use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PlanetViewResponse {
    pub planet_id:   i64,
    pub seed:        i64,
    pub subdivision: i64,
    pub planet_type: String,
    pub tiles:       Vec<TileDto>,
}

#[derive(Serialize)]
pub struct TileDto {
    pub id:               i64,
    pub face:             i64,
    pub u:                i64,
    pub v:                i64,
    pub tile_type:        String,
    pub yield_quality:    f64,
    pub rare_deposit:     Option<String>,
    pub owner_player_id:  Option<i64>,
    pub building:         Option<BuildingDto>,
    pub units:            Vec<UnitOnTileDto>,
}

#[derive(Serialize)]
pub struct BuildingDto {
    pub id:                     i64,
    pub building_type:          String,
    pub level:                  i64,
    pub hp:                     i64,
    pub max_hp:                 i64,
    pub under_attack:           bool,
    pub construction_done_tick: Option<i64>,
    pub flight_state:           Option<String>,
}

#[derive(Serialize)]
pub struct UnitOnTileDto {
    pub id:        i64,
    pub unit_type: String,
    pub count:     i64,
    pub hp:        i64,
    pub player_id: i64,
}

// Placed here (single source of truth), imported by both handler and service
#[derive(Deserialize)]
pub struct BuildRequest {
    pub face:          i64,
    pub u:             i64,
    pub v:             i64,
    pub building_type: String,
}
