/// Shared row structs used across multiple services.

#[derive(Debug, Clone)]
pub struct Player {
    pub id:       i64,
    pub username: String,
    pub email:    String,
}

#[derive(Debug, Clone)]
pub struct Planet {
    pub id:             i64,
    pub star_system_id: i64,
    pub seed:           i64,
    pub x:              f64,
    pub y:              f64,
    pub subdivision:    i64,
    pub planet_type:    String,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id:            i64,
    pub unit_type:     String,
    pub hp:            i64,
    pub max_hp:        i64,
    pub count:         i64,
    pub player_id:     i64,
    pub location_mode: String,
}
