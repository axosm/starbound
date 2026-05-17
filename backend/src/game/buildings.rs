pub struct BuildingDef {
    pub id:          &'static str,
    pub base_hp:     i64,
    pub build_ticks: i64,
    pub influence:   f64,
    pub can_fly:     bool,
}

const DEFS: &[BuildingDef] = &[
    BuildingDef { id: "town_center",    base_hp: 500, build_ticks: 5,  influence: 5.0,  can_fly: false },
    BuildingDef { id: "lumber_mill",    base_hp: 100, build_ticks: 3,  influence: 1.0,  can_fly: false },
    BuildingDef { id: "quarry",         base_hp: 100, build_ticks: 3,  influence: 1.0,  can_fly: false },
    BuildingDef { id: "farm",           base_hp:  80, build_ticks: 2,  influence: 0.5,  can_fly: false },
    BuildingDef { id: "water_pump",     base_hp:  80, build_ticks: 2,  influence: 0.5,  can_fly: false },
    BuildingDef { id: "mine",           base_hp: 150, build_ticks: 5,  influence: 1.5,  can_fly: false },
    BuildingDef { id: "barracks",       base_hp: 200, build_ticks: 8,  influence: 2.0,  can_fly: false },
    BuildingDef { id: "storage",        base_hp: 120, build_ticks: 4,  influence: 0.5,  can_fly: false },
    BuildingDef { id: "wall",           base_hp: 400, build_ticks: 6,  influence: 0.2,  can_fly: false },
    BuildingDef { id: "watchtower",     base_hp: 150, build_ticks: 4,  influence: 1.0,  can_fly: false },
    BuildingDef { id: "lab",            base_hp: 200, build_ticks: 10, influence: 2.0,  can_fly: false },
    BuildingDef { id: "launch_pad",     base_hp: 300, build_ticks: 15, influence: 3.0,  can_fly: false },
    BuildingDef { id: "space_dock",     base_hp: 400, build_ticks: 20, influence: 3.0,  can_fly: false },
    BuildingDef { id: "stargate",       base_hp: 600, build_ticks: 50, influence: 5.0,  can_fly: false },
    BuildingDef { id: "flying_fortress",base_hp: 300, build_ticks: 30, influence: 2.0,  can_fly: true  },
];

pub fn get_def(id: &str) -> Option<&'static BuildingDef> {
    DEFS.iter().find(|d| d.id == id)
}

pub fn all_defs() -> &'static [BuildingDef] { DEFS }
