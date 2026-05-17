/// Planet tile generation — creates tiles lazily when a player first
/// visits or claims an area of a planet.
///
/// Goldberg polyhedron (N=subdivision) gives ~10N² tiles total.
/// We store only explored tiles in the DB; the rest is generated on demand.

use crate::procgen;

pub struct GeneratedTile {
    pub face:          i64,
    pub u:             i64,
    pub v:             i64,
    pub tile_type:     &'static str,
    pub yield_quality: f64,
    pub rare_deposit:  Option<&'static str>,
}

const RARE_DEPOSITS: &[&str] = &[
    "coal","iron","gold","gems","petrol","uranium",
    "rare_earths","silicon","deuterium","dark_matter",
];

/// Generate all tiles for a face of a Goldberg polyhedron subdivision N.
/// For N=4 each face has ~16 tiles; total ~80 tiles per small planet.
pub fn generate_face(planet_seed: u64, face: i64, subdivision: i64) -> Vec<GeneratedTile> {
    let mut tiles = Vec::new();
    for u in 0..subdivision {
        for v in 0..subdivision {
            let tt    = procgen::tile_type_for(planet_seed, face, u, v);
            let yq    = procgen::yield_quality_for(planet_seed, face, u, v);
            let rare  = rare_deposit_for(planet_seed, face, u, v);
            tiles.push(GeneratedTile {
                face, u, v,
                tile_type:     tt,
                yield_quality: yq,
                rare_deposit:  rare,
            });
        }
    }
    tiles
}

fn rare_deposit_for(seed: u64, face: i64, u: i64, v: i64) -> Option<&'static str> {
    let hash = seed
        .wrapping_add(face as u64 * 1_111_111)
        .wrapping_add(u as u64 * 3_333_333)
        .wrapping_add(v as u64 * 7_777_777);
    // ~8% chance of a rare deposit
    if hash % 100 < 8 {
        Some(RARE_DEPOSITS[(hash as usize) % RARE_DEPOSITS.len()])
    } else {
        None
    }
}
