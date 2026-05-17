/// Pure combat math — no DB, no async.

pub struct UnitStats {
    pub attack:  i64,
    pub defense: i64,
    pub hp:      i64,
}

pub fn unit_stats(unit_type: &str) -> UnitStats {
    match unit_type {
        "soldier"    => UnitStats { attack: 10, defense: 5,  hp: 50  },
        "archer"     => UnitStats { attack: 15, defense: 3,  hp: 35  },
        "cavalry"    => UnitStats { attack: 20, defense: 8,  hp: 80  },
        "catapult"   => UnitStats { attack: 40, defense: 2,  hp: 60  },
        "fighter"    => UnitStats { attack: 30, defense: 10, hp: 100 },
        "bomber"     => UnitStats { attack: 60, defense: 5,  hp: 80  },
        "battleship" => UnitStats { attack: 80, defense: 20, hp: 200 },
        "transport"  => UnitStats { attack: 5,  defense: 5,  hp: 150 },
        _            => UnitStats { attack: 8,  defense: 4,  hp: 40  },
    }
}

/// Total damage dealt in one round by a unit formation.
/// count × base_attack, with slight randomisation baked out (deterministic).
pub fn unit_damage(unit_type: &str, count: i64) -> i64 {
    let stats = unit_stats(unit_type);
    stats.attack * count
}

/// Damage dealt to a building per round.
pub fn siege_damage(unit_type: &str, count: i64) -> i64 {
    let base = match unit_type {
        "catapult" => 80,
        "bomber"   => 120,
        _          => 5,
    };
    base * count
}
