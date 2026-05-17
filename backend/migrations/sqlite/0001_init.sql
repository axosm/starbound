PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─────────────────────────────────────────────────────────────
-- SERVER CONFIG  (speed multiplier stored here so it persists)
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS server_config (
  key    TEXT PRIMARY KEY,
  value  TEXT NOT NULL
);
INSERT OR IGNORE INTO server_config(key, value) VALUES
  ('game_speed',              '1'),
  ('allow_player_speed',      'true'),
  ('tick_ms',                 '60000'),
  ('game_tick',               '0');   -- monotonic in-game tick counter

-- ─────────────────────────────────────────────────────────────
-- 1. PLAYERS & AUTH
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS players (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  username      TEXT    NOT NULL,
  email         TEXT    NOT NULL UNIQUE,
  password_hash TEXT    NOT NULL,
  created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_login_at TEXT
);

-- ─────────────────────────────────────────────────────────────
-- 2. EMPIRES (alliances)
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS empires (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT    NOT NULL UNIQUE,
  created_by  INTEGER NOT NULL REFERENCES players(id),
  created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS empire_members (
  empire_id INTEGER NOT NULL REFERENCES empires(id),
  player_id INTEGER NOT NULL REFERENCES players(id),
  role      TEXT    NOT NULL DEFAULT 'member'
            CHECK(role IN ('leader','officer','member')),
  joined_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (empire_id, player_id)
);

-- ─────────────────────────────────────────────────────────────
-- 3. UNIVERSE
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS galaxies (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  seed       INTEGER NOT NULL,
  x          REAL    NOT NULL,
  y          REAL    NOT NULL,
  z          REAL    NOT NULL,
  created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(x, y, z)
);

CREATE TABLE IF NOT EXISTS star_systems (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  galaxy_id  INTEGER NOT NULL REFERENCES galaxies(id),
  seed       INTEGER NOT NULL,
  x          REAL    NOT NULL,
  y          REAL    NOT NULL,
  z          REAL    NOT NULL DEFAULT 0,
  created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(galaxy_id, x, y, z)
);
CREATE INDEX IF NOT EXISTS idx_star_systems_galaxy ON star_systems(galaxy_id);

CREATE TABLE IF NOT EXISTS planets (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  star_system_id  INTEGER NOT NULL REFERENCES star_systems(id),
  seed            INTEGER NOT NULL,
  orbit_index     INTEGER NOT NULL DEFAULT 0,  -- position in system (0 = closest)
  x               REAL    NOT NULL DEFAULT 0,  -- 2-D system-plane pos
  y               REAL    NOT NULL DEFAULT 0,
  subdivision     INTEGER NOT NULL DEFAULT 4,  -- Goldberg N
  planet_type     TEXT    NOT NULL DEFAULT 'terrestrial'
                  CHECK(planet_type IN ('terrestrial','ocean','desert','ice','lava','gas_giant','barren')),
  created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_planets_system ON planets(star_system_id);

-- Dynamic space objects
CREATE TABLE IF NOT EXISTS space_objects (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  star_system_id INTEGER NOT NULL REFERENCES star_systems(id),
  object_type    TEXT    NOT NULL
                 CHECK(object_type IN ('asteroid','pirate_base','anomaly','wreck','event','station','stargate')),
  owner_player_id INTEGER REFERENCES players(id),
  x              REAL    NOT NULL,
  y              REAL    NOT NULL,
  z              REAL    NOT NULL DEFAULT 0,
  properties     TEXT,   -- JSON
  hp             INTEGER NOT NULL DEFAULT 100,
  max_hp         INTEGER NOT NULL DEFAULT 100,
  cloaked        INTEGER NOT NULL DEFAULT 0,
  spawned_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  despawned_at   TEXT
);
CREATE INDEX IF NOT EXISTS idx_space_objects_system ON space_objects(star_system_id);

-- ─────────────────────────────────────────────────────────────
-- 4. TILES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS planet_tiles (
  id                      INTEGER PRIMARY KEY AUTOINCREMENT,
  planet_id               INTEGER NOT NULL REFERENCES planets(id),
  face                    INTEGER NOT NULL,
  u                       INTEGER NOT NULL,
  v                       INTEGER NOT NULL,
  tile_type               TEXT    NOT NULL
                          CHECK(tile_type IN (
                            'plains','forest','mountain','desert',
                            'snow','lava','water','ocean'
                          )),
  yield_quality           REAL    NOT NULL DEFAULT 0.5,
  rare_deposit            TEXT
                          CHECK(rare_deposit IN (
                            'coal','iron','gold','gems','petrol','uranium',
                            'rare_earths','silicon','deuterium','dark_matter',
                            NULL
                          )),
  owner_player_id         INTEGER REFERENCES players(id),
  influence_recalc_needed INTEGER NOT NULL DEFAULT 0,
  created_at              TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(planet_id, face, u, v)
);
CREATE INDEX IF NOT EXISTS idx_tiles_owner   ON planet_tiles(owner_player_id) WHERE owner_player_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tiles_recalc  ON planet_tiles(influence_recalc_needed) WHERE influence_recalc_needed = 1;

-- ─────────────────────────────────────────────────────────────
-- 5. INFLUENCE
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tile_influence (
  tile_id    INTEGER NOT NULL REFERENCES planet_tiles(id) ON DELETE CASCADE,
  player_id  INTEGER NOT NULL REFERENCES players(id)      ON DELETE CASCADE,
  score      REAL    NOT NULL DEFAULT 0,
  updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (tile_id, player_id)
);
CREATE INDEX IF NOT EXISTS idx_influence_tile   ON tile_influence(tile_id);
CREATE INDEX IF NOT EXISTS idx_influence_player ON tile_influence(player_id);

-- ─────────────────────────────────────────────────────────────
-- 6. BUILDINGS
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS buildings (
  id                   INTEGER PRIMARY KEY AUTOINCREMENT,
  player_id            INTEGER NOT NULL REFERENCES players(id),
  building_type        TEXT    NOT NULL,
  tile_id              INTEGER NOT NULL REFERENCES planet_tiles(id),
  level                INTEGER NOT NULL DEFAULT 1,
  hp                   INTEGER NOT NULL,
  max_hp               INTEGER NOT NULL,
  under_attack         INTEGER NOT NULL DEFAULT 0,
  destroyed_at         TEXT,
  can_fly              INTEGER NOT NULL DEFAULT 0,
  flight_state         TEXT    CHECK(flight_state IN ('grounded','lifting_off','flying','landing')),
  -- Construction: tick when done (NULL = already built)
  construction_done_tick INTEGER,
  created_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(tile_id)
);
CREATE INDEX IF NOT EXISTS idx_buildings_player       ON buildings(player_id);
CREATE INDEX IF NOT EXISTS idx_buildings_construction ON buildings(construction_done_tick) WHERE construction_done_tick IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_buildings_flying       ON buildings(flight_state)           WHERE flight_state != 'grounded';

-- ─────────────────────────────────────────────────────────────
-- 7. RESEARCH
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS player_research (
  player_id    INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  tech_id      TEXT    NOT NULL,
  level        INTEGER NOT NULL DEFAULT 0,
  -- Non-null while being researched
  research_done_tick INTEGER,
  PRIMARY KEY (player_id, tech_id)
);
CREATE INDEX IF NOT EXISTS idx_research_in_progress ON player_research(research_done_tick) WHERE research_done_tick IS NOT NULL;

-- ─────────────────────────────────────────────────────────────
-- 8. UNITS & MOVEMENT
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS units (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_type      TEXT    NOT NULL,
  is_squad       INTEGER NOT NULL DEFAULT 0,
  count          INTEGER NOT NULL DEFAULT 1,
  hp             INTEGER NOT NULL,
  max_hp         INTEGER NOT NULL DEFAULT 100,
  player_id      INTEGER NOT NULL REFERENCES players(id),
  in_battle      INTEGER NOT NULL DEFAULT 0,
  cloaked        INTEGER NOT NULL DEFAULT 0,

  location_mode  TEXT    NOT NULL DEFAULT 'planet_surface'
                 CHECK(location_mode IN ('planet_surface','in_orbit','in_space','embarked')),

  -- Surface
  planet_id      INTEGER REFERENCES planets(id),
  planet_face    INTEGER,
  planet_u       INTEGER,
  planet_v       INTEGER,

  -- Orbit
  orbit_planet_id INTEGER REFERENCES planets(id),
  orbit_altitude  TEXT    CHECK(orbit_altitude IN ('low','high')),

  -- Space
  star_system_id INTEGER REFERENCES star_systems(id),
  space_x        REAL,
  space_y        REAL,
  space_z        REAL DEFAULT 0,

  -- Cargo capacity (for transport ships)
  cargo_capacity INTEGER NOT NULL DEFAULT 0,

  customization  TEXT,  -- JSON
  created_at     TEXT   NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_units_player     ON units(player_id);
CREATE INDEX IF NOT EXISTS idx_units_planet     ON units(planet_id, planet_face, planet_u, planet_v) WHERE location_mode = 'planet_surface';
CREATE INDEX IF NOT EXISTS idx_units_orbit      ON units(orbit_planet_id) WHERE location_mode = 'in_orbit';
CREATE INDEX IF NOT EXISTS idx_units_space      ON units(star_system_id, space_x, space_y, space_z) WHERE location_mode = 'in_space';

CREATE TABLE IF NOT EXISTS move_orders (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_id      INTEGER REFERENCES units(id),
  building_id  INTEGER REFERENCES buildings(id),
  mover_type   TEXT    NOT NULL CHECK(mover_type IN ('unit','building')),

  move_type    TEXT    NOT NULL CHECK(move_type IN (
                 'tile_walk','launch_to_orbit','orbit_to_space',
                 'space_travel','enter_orbit','land',
                 'building_liftoff','building_land','loot_and_retreat',
                 'hyperjump'
               )),

  from_planet_id   INTEGER REFERENCES planets(id),
  from_planet_face INTEGER,
  from_planet_u    INTEGER,
  from_planet_v    INTEGER,

  to_planet_id     INTEGER REFERENCES planets(id),
  to_planet_face   INTEGER,
  to_planet_u      INTEGER,
  to_planet_v      INTEGER,

  from_star_system_id INTEGER REFERENCES star_systems(id),
  from_space_x    REAL,
  from_space_y    REAL,
  from_space_z    REAL DEFAULT 0,

  to_star_system_id INTEGER REFERENCES star_systems(id),
  to_space_x      REAL,
  to_space_y      REAL,
  to_space_z      REAL DEFAULT 0,

  -- stored in GAME TICKS (not wall-clock time)
  start_tick      INTEGER NOT NULL,
  arrival_tick    INTEGER NOT NULL,

  FOREIGN KEY(unit_id) REFERENCES units(id)
);
CREATE INDEX IF NOT EXISTS idx_move_orders_unit    ON move_orders(unit_id)    WHERE unit_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_move_orders_building ON move_orders(building_id) WHERE building_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_move_orders_arrival ON move_orders(arrival_tick);

CREATE TABLE IF NOT EXISTS retreat_orders (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_id         INTEGER NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  player_id       INTEGER NOT NULL REFERENCES players(id),
  retreat_to_face INTEGER NOT NULL,
  retreat_to_u    INTEGER NOT NULL,
  retreat_to_v    INTEGER NOT NULL,
  created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_retreat_unit   ON retreat_orders(unit_id);
CREATE INDEX IF NOT EXISTS idx_retreat_player ON retreat_orders(player_id);

-- ─────────────────────────────────────────────────────────────
-- 9. COMBAT
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS battles (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  tile_id       INTEGER REFERENCES planet_tiles(id),
  star_system_id INTEGER REFERENCES star_systems(id),
  space_x       REAL,
  space_y       REAL,
  space_z       REAL,
  attacker_id   INTEGER NOT NULL REFERENCES players(id),
  defender_id   INTEGER NOT NULL REFERENCES players(id),
  phase         TEXT    NOT NULL DEFAULT 'vs_units'
                CHECK(phase IN ('vs_units','vs_building')),
  started_tick  INTEGER NOT NULL DEFAULT 0,
  last_tick     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_battles_tile     ON battles(tile_id)      WHERE tile_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_battles_attacker ON battles(attacker_id);
CREATE INDEX IF NOT EXISTS idx_battles_defender ON battles(defender_id);

CREATE TABLE IF NOT EXISTS battle_reports (
  id                       INTEGER PRIMARY KEY AUTOINCREMENT,
  battle_id                INTEGER NOT NULL,
  tile_id                  INTEGER REFERENCES planet_tiles(id),
  star_system_id           INTEGER REFERENCES star_systems(id),
  space_x                  REAL,
  space_y                  REAL,
  space_z                  REAL,
  attacker_id              INTEGER NOT NULL REFERENCES players(id),
  defender_id              INTEGER NOT NULL REFERENCES players(id),
  outcome                  TEXT    NOT NULL
                           CHECK(outcome IN (
                             'attacker_victory','defender_victory',
                             'attacker_retreated','defender_retreated',
                             'attacker_looted','draw'
                           )),
  attacker_units_snapshot  TEXT    NOT NULL,  -- JSON
  defender_units_snapshot  TEXT    NOT NULL,  -- JSON
  resources_looted_json    TEXT,
  started_tick             INTEGER NOT NULL,
  ended_tick               INTEGER NOT NULL,
  attacker_read            INTEGER NOT NULL DEFAULT 0,
  defender_read            INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_reports_attacker ON battle_reports(attacker_id, attacker_read);
CREATE INDEX IF NOT EXISTS idx_reports_defender ON battle_reports(defender_id, defender_read);

-- ─────────────────────────────────────────────────────────────
-- 10. RESOURCES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS player_resources (
  player_id     INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  resource_type TEXT    NOT NULL
                CHECK(resource_type IN (
                  'wood','stone','food','water',
                  'coal','iron','petrol','copper',
                  'silicon','uranium','rare_earths','electricity',
                  'deuterium','dark_matter','titanium','antimatter'
                )),
  amount        REAL    NOT NULL DEFAULT 0,
  cap           REAL    NOT NULL DEFAULT 1000,
  updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (player_id, resource_type)
);
CREATE INDEX IF NOT EXISTS idx_player_resources ON player_resources(player_id);

CREATE TABLE IF NOT EXISTS unit_cargo (
  unit_id       INTEGER NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  resource_type TEXT    NOT NULL,
  amount        REAL    NOT NULL DEFAULT 0,
  PRIMARY KEY (unit_id, resource_type)
);

-- ─────────────────────────────────────────────────────────────
-- 11. SERVER-SENT EVENTS LOG
-- Used so polling clients can fetch alerts they missed
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sse_events (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  player_id   INTEGER REFERENCES players(id) ON DELETE CASCADE,  -- NULL = broadcast
  event_type  TEXT    NOT NULL,
  payload     TEXT    NOT NULL,  -- JSON
  created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  expires_at  TEXT    NOT NULL  -- auto-purged after 24h
);
CREATE INDEX IF NOT EXISTS idx_sse_player ON sse_events(player_id, created_at);
