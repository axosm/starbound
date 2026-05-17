/// Influence calculation — determines tile ownership.
///
/// Score for player P on tile T =
///   SUM over all of P's buildings: influence_power / distance(building, T)
///   where distance <= influence_radius of that building.
///
/// The player with the highest score owns the tile.
/// Ties go to the existing owner (defender advantage).

use crate::game::buildings;

pub struct InfluenceSource {
    pub face:    i64,
    pub u:       i64,
    pub v:       i64,
    pub btype:   String,
    pub level:   i64,
}

pub fn hex_distance(au: i64, av: i64, bu: i64, bv: i64) -> f64 {
    // Offset hex distance approximation (same face assumed)
    let du = (au - bu) as f64;
    let dv = (av - bv) as f64;
    (du * du + dv * dv).sqrt()
}

pub fn score_for(
    tile_u:   i64,
    tile_v:   i64,
    sources:  &[InfluenceSource],
) -> f64 {
    let mut total = 0.0_f64;
    for src in sources {
        let def = match buildings::get_def(&src.btype) {
            Some(d) => d,
            None    => continue,
        };
        let power  = def.influence * src.level as f64;
        let radius = 2.0 + src.level as f64;   // influence radius in tiles
        let dist   = hex_distance(src.u, src.v, tile_u, tile_v).max(0.5);
        if dist <= radius {
            total += power / dist;
        }
    }
    total
}
