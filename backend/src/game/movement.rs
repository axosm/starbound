/// Movement helpers — no DB, no async.

/// Duration in game ticks for each move type.
pub fn travel_ticks(move_type: &str) -> i64 {
    match move_type {
        "tile_walk"       => 1,
        "launch_to_orbit" => 2,
        "orbit_to_space"  => 3,
        "space_travel"    => 10,
        "enter_orbit"     => 2,
        "land"            => 2,
        "building_liftoff"=> 3,
        "building_land"   => 3,
        "loot_and_retreat"=> 4,
        "hyperjump"       => 30,
        _                 => 5,
    }
}

/// Given a completed move order, return the new unit state.
/// Returns: (location_mode, planet_id, face, u, v, orbit_planet_id,
///           star_system_id, space_x, space_y, space_z)
#[allow(clippy::too_many_arguments)]
pub fn resolve_destination(
    move_type:    &str,
    to_planet_id: Option<i64>,
    to_face:      Option<i64>,
    to_u:         Option<i64>,
    to_v:         Option<i64>,
    to_system_id: Option<i64>,
    to_x:         Option<f64>,
    to_y:         Option<f64>,
    to_z:         Option<f64>,
) -> (
    String,
    Option<i64>, Option<i64>, Option<i64>, Option<i64>,
    Option<i64>,
    Option<i64>, Option<f64>, Option<f64>, Option<f64>,
) {
    match move_type {
        "tile_walk" | "land" | "building_land" => (
            "planet_surface".into(),
            to_planet_id, to_face, to_u, to_v,
            None, None, None, None, None,
        ),
        "launch_to_orbit" | "enter_orbit" => (
            "in_orbit".into(),
            None, None, None, None,
            to_planet_id, None, None, None, None,
        ),
        "orbit_to_space" | "space_travel" | "hyperjump" => (
            "in_space".into(),
            None, None, None, None,
            None, to_system_id, to_x, to_y, to_z,
        ),
        _ => (
            "planet_surface".into(),
            to_planet_id, to_face, to_u, to_v,
            None, None, None, None, None,
        ),
    }
}
