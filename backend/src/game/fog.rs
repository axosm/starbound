/// Fog of war — which tiles are visible to a given player.
/// 
/// Visibility rules:
///   - A player can see any tile they own.
///   - A player can see tiles within range of their units/buildings.
///   - Range is based on unit type / building type.
///   - Space units reveal a radius around their position.

use std::collections::HashSet;

/// Tile coordinate
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct TileCoord { pub face: i64, pub u: i64, pub v: i64 }

/// Given a set of owned/occupied tile coords, return all visible tiles
/// within a given hex radius (Manhattan distance on hex grid).
pub fn visible_tiles(
    occupied: &[TileCoord],
    sight_range: i64,
) -> HashSet<TileCoord> {
    let mut visible = HashSet::new();
    for origin in occupied {
        for du in -sight_range..=sight_range {
            for dv in -sight_range..=sight_range {
                if du.abs() + dv.abs() <= sight_range {
                    visible.insert(TileCoord {
                        face: origin.face,
                        u:    origin.u + du,
                        v:    origin.v + dv,
                    });
                }
            }
        }
    }
    visible
}

/// Default sight range per building type.
pub fn building_sight(building_type: &str) -> i64 {
    match building_type {
        "watchtower" => 4,
        "town_center"=> 3,
        "wall"       => 2,
        _            => 1,
    }
}

/// Default sight range per unit type.
pub fn unit_sight(unit_type: &str) -> i64 {
    match unit_type {
        "archer"   => 3,
        "cavalry"  => 4,
        "fighter"  => 5,
        _          => 2,
    }
}
