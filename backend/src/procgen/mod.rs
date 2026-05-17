use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand::Rng;

pub fn seeded_rng(seed: u64) -> Pcg64 {
    Pcg64::seed_from_u64(seed)
}

pub fn rand_f64(rng: &mut Pcg64, min: f64, max: f64) -> f64 {
    rng.gen_range(min..=max)
}

pub fn rand_i64(rng: &mut Pcg64, min: i64, max: i64) -> i64 {
    rng.gen_range(min..=max)
}

/// Pick a tile type based on a seed + position, deterministically.
pub fn tile_type_for(seed: u64, face: i64, u: i64, v: i64) -> &'static str {
    let hash = seed
        .wrapping_add(face as u64 * 1_000_003)
        .wrapping_add(u as u64 * 9_999_991)
        .wrapping_add(v as u64 * 7_777_777);
    match hash % 8 {
        0 => "plains",
        1 => "forest",
        2 => "mountain",
        3 => "desert",
        4 => "snow",
        5 => "lava",
        6 => "water",
        _ => "ocean",
    }
}

/// Yield quality [0.3, 1.0] for a tile, deterministic from seed + position.
pub fn yield_quality_for(seed: u64, face: i64, u: i64, v: i64) -> f64 {
    let hash = seed
        .wrapping_add(face as u64 * 2_000_003)
        .wrapping_add(u as u64 * 3_000_007)
        .wrapping_add(v as u64 * 4_000_011);
    0.3 + (hash % 1000) as f64 / 1000.0 * 0.7
}
