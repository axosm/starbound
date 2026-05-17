-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- ─────────────────────────────────────────────────────────────
-- SERVER CONFIG
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS server_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
INSERT INTO server_config(key, value) VALUES
  ('game_speed',         '1'),
  ('allow_player_speed', 'true'),
  ('tick_ms',            '60000'),
  ('game_tick',          '0')
ON CONFLICT DO NOTHING;

-- ─────────────────────────────────────────────────────────────
-- 1. PLAYERS
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS players (
  id            BIGSERIAL PRIMARY KEY,
  username      TEXT      NOT NULL,
  email         TEXT      NOT NULL UNIQUE,
  password_hash TEXT      NOT NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_login_at TIMESTAMPTZ
);

-- ─────────────────────────────────────────────────────────────
-- 2. EMPIRES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS empires (
  id         BIGSERIAL PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE,
  created_by BIGINT NOT NULL REFERENCES players(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS empire_members (
  empire_id BIGINT NOT NULL REFERENCES empires(id),
  player_id BIGINT NOT NULL REFERENCES players(id),
  role      TEXT   NOT NULL DEFAULT 'member'
            CHECK(role IN ('leader','officer','member')),
  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (empire_id, player_id)
);

-- ─────────────────────────────────────────────────────────────
-- 3. UNIVERSE
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS galaxies (
  id         BIGSERIAL PRIMARY KEY,
  seed       BIGINT NOT NULL,
  pos        geometry(PointZ, 0) NOT NULL,  -- (x,y,z) in universe space
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_galaxies_pos ON galaxies USING GIST(pos);

CREATE TABLE IF NOT EXISTS star_systems (
  id         BIGSERIAL PRIMARY KEY,
  galaxy_id  BIGINT NOT NULL REFERENCES galaxies(id),
  seed       BIGINT NOT NULL,
  -- Position within the galaxy — PostGIS 3D point
  pos        geometry(PointZ, 0) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(galaxy_id, pos)
);
CREATE INDEX IF NOT EXISTS idx_star_systems_galaxy ON star_systems(galaxy_id);
CREATE INDEX IF NOT EXISTS idx_star_systems_pos    ON star_systems USING GIST(pos);

CREATE TABLE IF NOT EXISTS planets (
  id             BIGSERIAL PRIMARY KEY,
  star_system_id BIGINT NOT NULL REFERENCES star_systems(id),
  seed           BIGINT NOT NULL,
  orbit_index    INT    NOT NULL DEFAULT 0,
  -- 2-D position within the solar system plane
  pos            geometry(Point, 0) NOT NULL,
  subdivision    INT    NOT NULL DEFAULT 4,
  planet_type    TEXT   NOT NULL DEFAULT 'terrestrial'
                 CHECK(planet_type IN ('terrestrial','ocean','desert','ice','lava','gas_giant','barren')),
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_planets_system ON planets(star_system_id);
CREATE INDEX IF NOT EXISTS idx_planets_pos    ON planets USING GIST(pos);

CREATE TABLE IF NOT EXISTS space_objects (
  id              BIGSERIAL PRIMARY KEY,
  star_system_id  BIGINT NOT NULL REFERENCES star_systems(id),
  object_type     TEXT   NOT NULL
                  CHECK(object_type IN ('asteroid','pirate_base','anomaly','wreck','event','station','stargate')),
  owner_player_id BIGINT REFERENCES players(id),
  -- 3-D position in system space — used for proximity queries
  pos             geometry(PointZ, 0) NOT NULL,
  properties      JSONB,
  hp              INT    NOT NULL DEFAULT 100,
  max_hp          INT    NOT NULL DEFAULT 100,
  cloaked         BOOLEAN NOT NULL DEFAULT FALSE,
  spawned_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  despawned_at    TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_space_objects_system ON space_objects(star_system_id);
CREATE INDEX IF NOT EXISTS idx_space_objects_pos    ON space_objects USING GIST(pos);

-- ─────────────────────────────────────────────────────────────
-- 4. TILES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS planet_tiles (
  id                      BIGSERIAL PRIMARY KEY,
  planet_id               BIGINT NOT NULL REFERENCES planets(id),
  face                    INT    NOT NULL,
  u                       INT    NOT NULL,
  v                       INT    NOT NULL,
  tile_type               TEXT   NOT NULL
                          CHECK(tile_type IN (
                            'plains','forest','mountain','desert',
                            'snow','lava','water','ocean'
                          )),
  yield_quality           REAL   NOT NULL DEFAULT 0.5,
  rare_deposit            TEXT
                          CHECK(rare_deposit IN (
                            'coal','iron','gold','gems','petrol','uranium',
                            'rare_earths','silicon','deuterium','dark_matter',
                            NULL
                          )),
  owner_player_id         BIGINT REFERENCES players(id),
  influence_recalc_needed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(planet_id, face, u, v)
);
CREATE INDEX IF NOT EXISTS idx_tiles_owner  ON planet_tiles(owner_player_id) WHERE owner_player_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tiles_recalc ON planet_tiles(influence_recalc_needed) WHERE influence_recalc_needed = TRUE;

-- ─────────────────────────────────────────────────────────────
-- 5. INFLUENCE
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tile_influence (
  tile_id    BIGINT NOT NULL REFERENCES planet_tiles(id) ON DELETE CASCADE,
  player_id  BIGINT NOT NULL REFERENCES players(id)      ON DELETE CASCADE,
  score      REAL   NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tile_id, player_id)
);
CREATE INDEX IF NOT EXISTS idx_influence_tile   ON tile_influence(tile_id);
CREATE INDEX IF NOT EXISTS idx_influence_player ON tile_influence(player_id);

-- ─────────────────────────────────────────────────────────────
-- 6. BUILDINGS
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS buildings (
  id                      BIGSERIAL PRIMARY KEY,
  player_id               BIGINT NOT NULL REFERENCES players(id),
  building_type           TEXT   NOT NULL,
  tile_id                 BIGINT NOT NULL REFERENCES planet_tiles(id),
  level                   INT    NOT NULL DEFAULT 1,
  hp                      INT    NOT NULL,
  max_hp                  INT    NOT NULL,
  under_attack            BOOLEAN NOT NULL DEFAULT FALSE,
  destroyed_at            TIMESTAMPTZ,
  can_fly                 BOOLEAN NOT NULL DEFAULT FALSE,
  flight_state            TEXT    CHECK(flight_state IN ('grounded','lifting_off','flying','landing')),
  construction_done_tick  BIGINT,
  created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(tile_id)
);
CREATE INDEX IF NOT EXISTS idx_buildings_player       ON buildings(player_id);
CREATE INDEX IF NOT EXISTS idx_buildings_construction ON buildings(construction_done_tick) WHERE construction_done_tick IS NOT NULL;

-- ─────────────────────────────────────────────────────────────
-- 7. RESEARCH
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS player_research (
  player_id          BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  tech_id            TEXT   NOT NULL,
  level              INT    NOT NULL DEFAULT 0,
  research_done_tick BIGINT,
  PRIMARY KEY (player_id, tech_id)
);
CREATE INDEX IF NOT EXISTS idx_research_in_progress ON player_research(research_done_tick) WHERE research_done_tick IS NOT NULL;

-- ─────────────────────────────────────────────────────────────
-- 8. UNITS & MOVEMENT
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS units (
  id              BIGSERIAL PRIMARY KEY,
  unit_type       TEXT    NOT NULL,
  is_squad        BOOLEAN NOT NULL DEFAULT FALSE,
  count           INT     NOT NULL DEFAULT 1,
  hp              INT     NOT NULL,
  max_hp          INT     NOT NULL DEFAULT 100,
  player_id       BIGINT  NOT NULL REFERENCES players(id),
  in_battle       BOOLEAN NOT NULL DEFAULT FALSE,
  cloaked         BOOLEAN NOT NULL DEFAULT FALSE,

  location_mode   TEXT    NOT NULL DEFAULT 'planet_surface'
                  CHECK(location_mode IN ('planet_surface','in_orbit','in_space','embarked')),

  planet_id       BIGINT  REFERENCES planets(id),
  planet_face     INT,
  planet_u        INT,
  planet_v        INT,

  orbit_planet_id BIGINT  REFERENCES planets(id),
  orbit_altitude  TEXT    CHECK(orbit_altitude IN ('low','high')),

  star_system_id  BIGINT  REFERENCES star_systems(id),
  -- PostGIS 3-D point for space units — enables ST_DWithin proximity queries
  space_pos       geometry(PointZ, 0),

  cargo_capacity  INT     NOT NULL DEFAULT 0,
  customization   JSONB,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_units_player ON units(player_id);
CREATE INDEX IF NOT EXISTS idx_units_planet ON units(planet_id, planet_face, planet_u, planet_v) WHERE location_mode = 'planet_surface';
CREATE INDEX IF NOT EXISTS idx_units_orbit  ON units(orbit_planet_id) WHERE location_mode = 'in_orbit';
-- GiST index for 3-D proximity searches (ST_DWithin) in space
CREATE INDEX IF NOT EXISTS idx_units_space_gist ON units USING GIST(space_pos) WHERE location_mode = 'in_space';

CREATE TABLE IF NOT EXISTS move_orders (
  id           BIGSERIAL PRIMARY KEY,
  unit_id      BIGINT REFERENCES units(id),
  building_id  BIGINT REFERENCES buildings(id),
  mover_type   TEXT   NOT NULL CHECK(mover_type IN ('unit','building')),
  move_type    TEXT   NOT NULL CHECK(move_type IN (
                 'tile_walk','launch_to_orbit','orbit_to_space',
                 'space_travel','enter_orbit','land',
                 'building_liftoff','building_land','loot_and_retreat','hyperjump'
               )),

  from_planet_id   BIGINT REFERENCES planets(id),
  from_planet_face INT,
  from_planet_u    INT,
  from_planet_v    INT,

  to_planet_id     BIGINT REFERENCES planets(id),
  to_planet_face   INT,
  to_planet_u      INT,
  to_planet_v      INT,

  from_star_system_id BIGINT REFERENCES star_systems(id),
  from_pos        geometry(PointZ, 0),

  to_star_system_id BIGINT REFERENCES star_systems(id),
  to_pos          geometry(PointZ, 0),

  start_tick   BIGINT NOT NULL,
  arrival_tick BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_move_orders_unit    ON move_orders(unit_id)     WHERE unit_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_move_orders_arrival ON move_orders(arrival_tick);

CREATE TABLE IF NOT EXISTS retreat_orders (
  id              BIGSERIAL PRIMARY KEY,
  unit_id         BIGINT NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  player_id       BIGINT NOT NULL REFERENCES players(id),
  retreat_to_face INT    NOT NULL,
  retreat_to_u    INT    NOT NULL,
  retreat_to_v    INT    NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ─────────────────────────────────────────────────────────────
-- 9. COMBAT
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS battles (
  id             BIGSERIAL PRIMARY KEY,
  tile_id        BIGINT REFERENCES planet_tiles(id),
  star_system_id BIGINT REFERENCES star_systems(id),
  space_pos      geometry(PointZ, 0),
  attacker_id    BIGINT NOT NULL REFERENCES players(id),
  defender_id    BIGINT NOT NULL REFERENCES players(id),
  phase          TEXT   NOT NULL DEFAULT 'vs_units'
                 CHECK(phase IN ('vs_units','vs_building')),
  started_tick   BIGINT NOT NULL DEFAULT 0,
  last_tick      BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_battles_attacker ON battles(attacker_id);
CREATE INDEX IF NOT EXISTS idx_battles_defender ON battles(defender_id);
-- Exclusion constraint: only one active battle per tile at a time
-- (requires btree_gist)
CREATE UNIQUE INDEX IF NOT EXISTS idx_battles_tile_unique
  ON battles(tile_id) WHERE tile_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS battle_reports (
  id                      BIGSERIAL PRIMARY KEY,
  battle_id               BIGINT NOT NULL,
  tile_id                 BIGINT REFERENCES planet_tiles(id),
  star_system_id          BIGINT REFERENCES star_systems(id),
  space_pos               geometry(PointZ, 0),
  attacker_id             BIGINT NOT NULL REFERENCES players(id),
  defender_id             BIGINT NOT NULL REFERENCES players(id),
  outcome                 TEXT   NOT NULL
                          CHECK(outcome IN (
                            'attacker_victory','defender_victory',
                            'attacker_retreated','defender_retreated',
                            'attacker_looted','draw'
                          )),
  attacker_units_snapshot JSONB  NOT NULL,
  defender_units_snapshot JSONB  NOT NULL,
  resources_looted_json   JSONB,
  started_tick            BIGINT NOT NULL,
  ended_tick              BIGINT NOT NULL,
  attacker_read           BOOLEAN NOT NULL DEFAULT FALSE,
  defender_read           BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX IF NOT EXISTS idx_reports_attacker ON battle_reports(attacker_id, attacker_read);
CREATE INDEX IF NOT EXISTS idx_reports_defender ON battle_reports(defender_id, defender_read);

-- ─────────────────────────────────────────────────────────────
-- 10. RESOURCES
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS player_resources (
  player_id     BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  resource_type TEXT   NOT NULL
                CHECK(resource_type IN (
                  'wood','stone','food','water',
                  'coal','iron','petrol','copper',
                  'silicon','uranium','rare_earths','electricity',
                  'deuterium','dark_matter','titanium','antimatter'
                )),
  amount        REAL   NOT NULL DEFAULT 0,
  cap           REAL   NOT NULL DEFAULT 1000,
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (player_id, resource_type)
);

CREATE TABLE IF NOT EXISTS unit_cargo (
  unit_id       BIGINT NOT NULL REFERENCES units(id) ON DELETE CASCADE,
  resource_type TEXT   NOT NULL,
  amount        REAL   NOT NULL DEFAULT 0,
  PRIMARY KEY (unit_id, resource_type)
);

-- ─────────────────────────────────────────────────────────────
-- 11. SSE EVENT LOG
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sse_events (
  id          BIGSERIAL PRIMARY KEY,
  player_id   BIGINT REFERENCES players(id) ON DELETE CASCADE,
  event_type  TEXT   NOT NULL,
  payload     JSONB  NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at  TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sse_player ON sse_events(player_id, created_at);
