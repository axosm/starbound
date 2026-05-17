export interface LoginResponse {
  token:     string;
  player_id: number;
  username:  string;
}

export interface SpeedResponse {
  speed:                     number;
  real_tick_ms:              number;
  game_tick_ms:              number;
  allow_player_speed_change: boolean;
  current_tick:              number;
}

export interface GameInitResponse {
  player_id:    number;
  username:     string;
  tick:         number;
  speed:        number;
  game_tick_ms: number;
  real_tick_ms: number;
  home_planet:  PlanetSummary | null;
}

export interface PlanetSummary {
  id:             number;
  star_system_id: number;
  seed:           number;
  x:              number;
  y:              number;
  subdivision:    number;
  planet_type:    string;
}

export interface PlanetViewResponse {
  planet_id:   number;
  seed:        number;
  subdivision: number;
  planet_type: string;
  tiles:       TileDto[];
}

export interface TileDto {
  id:               number;
  face:             number;
  u:                number;
  v:                number;
  tile_type:        string;
  yield_quality:    number;
  rare_deposit:     string | null;
  owner_player_id:  number | null;
  building:         BuildingDto | null;
  units:            UnitOnTileDto[];
}

export interface BuildingDto {
  id:                     number;
  building_type:          string;
  level:                  number;
  hp:                     number;
  max_hp:                 number;
  under_attack:           boolean;
  construction_done_tick: number | null;
  flight_state:           string | null;
}

export interface UnitOnTileDto {
  id:        number;
  unit_type: string;
  count:     number;
  hp:        number;
  player_id: number;
}

export interface UnitDto {
  id:              number;
  unit_type:       string;
  hp:              number;
  max_hp:          number;
  count:           number;
  location_mode:   string;
  planet_id:       number | null;
  planet_face:     number | null;
  planet_u:        number | null;
  planet_v:        number | null;
  orbit_planet_id: number | null;
  star_system_id:  number | null;
  space_x:         number | null;
  space_y:         number | null;
  space_z:         number | null;
  in_battle:       boolean;
  move_order:      MoveOrderDto | null;
}

export interface MoveOrderDto {
  order_id:     number;
  move_type:    string;
  start_tick:   number;
  arrival_tick: number;
  ticks_left:   number;
  eta_seconds:  number;
}

export interface ResourceEntry {
  resource_type: string;
  amount:        number;
  cap:           number;
  rate_per_tick: number;
}

export interface ResourcesResponse {
  resources: ResourceEntry[];
  tick:      number;
  speed:     number;
}

// SSE event shapes
export interface SseTickEvent {
  tick:         number;
  speed:        number;
  real_tick_ms: number;
}

export interface SseSpeedEvent {
  speed:        number;
  real_tick_ms: number;
  game_tick_ms: number;
  current_tick: number;
}

export interface SseBattleEvent {
  planet_id:   number;
  face:        number;
  u:           number;
  v:           number;
  attacker_id: number;
  defender_id: number;
}
