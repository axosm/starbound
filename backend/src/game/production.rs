/// Production rate calculations — no DB, no async.

pub fn base_rate(resource_type: &str, building_level: i64) -> f64 {
    let base = match resource_type {
        "wood"        => 2.0,
        "stone"       => 1.5,
        "food"        => 1.0,
        "water"       => 1.0,
        "coal"        => 0.8,
        "iron"        => 0.6,
        "petrol"      => 0.4,
        "copper"      => 0.5,
        "silicon"     => 0.3,
        "uranium"     => 0.1,
        "rare_earths" => 0.05,
        "electricity" => 1.2,
        "deuterium"   => 0.08,
        "dark_matter" => 0.02,
        "titanium"    => 0.2,
        "antimatter"  => 0.01,
        _             => 0.0,
    };
    base * (building_level as f64)
}

/// Multiply base rate by yield quality of the tile (0.0–1.0).
pub fn tile_rate(resource_type: &str, building_level: i64, yield_quality: f64) -> f64 {
    base_rate(resource_type, building_level) * yield_quality
}

/// Storage cap contributed by one storage building at a given level.
pub fn storage_cap(building_level: i64) -> f64 {
    1000.0 * building_level as f64
}
