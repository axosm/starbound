import type {
  GameInitResponse, LoginResponse, PlanetViewResponse,
  ResourcesResponse, SpeedResponse, UnitDto,
} from '../types/api';

const BASE = '/api';
let _token: string | null = null;

export function setToken(t: string)       { _token = t; }
export function getToken(): string | null { return _token; }

async function req<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (_token) headers['Authorization'] = `Bearer ${_token}`;
  const res = await fetch(`${BASE}${path}`, {
    method, headers, body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? res.statusText);
  }
  return res.json() as Promise<T>;
}

// ── Auth ──────────────────────────────────────────────────────
export const register = (username: string, email: string, password: string) =>
  req<LoginResponse>('POST', '/auth/register', { username, email, password });
export const login = (email: string, password: string) =>
  req<LoginResponse>('POST', '/auth/login', { email, password });

// ── Game ──────────────────────────────────────────────────────
export const gameInit = () => req<GameInitResponse>('GET', '/game/init');

// ── Speed ─────────────────────────────────────────────────────
export const getSpeed = () => req<SpeedResponse>('GET', '/speed');
export const setSpeed = (speed: number) => req<SpeedResponse>('POST', '/speed', { speed });

// ── Planet ────────────────────────────────────────────────────
export const getPlanet = (id: number) => req<PlanetViewResponse>('GET', `/planets/${id}`);
export const buildOnTile = (planetId: number, face: number, u: number, v: number, buildingType: string) =>
  req<{ ok: boolean }>('POST', `/planets/${planetId}/build`, { face, u, v, building_type: buildingType });

// ── Units ─────────────────────────────────────────────────────
export const listUnits = ()    => req<UnitDto[]>('GET', '/units');
export const getUnitDefs = ()  => req<{ unit_defs: UnitDefDto[] }>('GET', '/units/defs');
export const recruitUnit = (body: {
  unit_type: string; count: number;
  planet_id: number; face: number; u: number; v: number;
}) => req('POST', '/units/recruit', body);
export const moveUnit = (body: {
  unit_id: number; move_type: string;
  to_planet_id?: number; to_face?: number; to_u?: number; to_v?: number;
  to_system_id?: number; to_x?: number; to_y?: number; to_z?: number;
}) => req('POST', '/units/move', body);

// ── Space ─────────────────────────────────────────────────────
export const spaceScan = (scanner_unit_id: number, radius: number) =>
  req('POST', '/space/scan', { scanner_unit_id, radius });
export const setCloak = (unit_id: number, cloak: boolean) =>
  req('POST', '/space/cloak', { unit_id, cloak });
export const getBattleReports = (limit = 20) =>
  req<{ reports: BattleReportDto[] }>('GET', `/space/reports?limit=${limit}`);
export const markReportsRead = () => req('POST', '/space/reports/read', {});

// ── Research ──────────────────────────────────────────────────
export const getResearch = () =>
  req<{ techs: TechDto[] }>('GET', '/research');
export const startResearch = (tech_id: string) =>
  req<{ tech: TechDto }>('POST', '/research/start', { tech_id });

// ── Resources ─────────────────────────────────────────────────
export const getResources = () => req<ResourcesResponse>('GET', '/resources');

// ── Empire ────────────────────────────────────────────────────
export const getMyEmpire    = ()           => req('GET',    '/empire');
export const createEmpire   = (name: string) => req('POST', '/empire', { name });
export const leaveEmpire    = ()           => req('POST',   '/empire/leave', {});
export const inviteToEmpire = (id: number, target_username: string) =>
  req('POST', `/empire/${id}/invite`, { target_username });
export const kickFromEmpire = (id: number, pid: number) =>
  req('DELETE', `/empire/${id}/kick/${pid}`, {});


// ── Universe ──────────────────────────────────────────────────
export const listSystems = (galaxy_id?: number) =>
  req<{ systems: { id: number; seed: number; x: number; y: number; z: number }[] }>(
    'GET', `/systems${galaxy_id !== undefined ? `?galaxy_id=${galaxy_id}` : ''}`
  );

export const listSystemPlanets = (system_id: number) =>
  req<{ planets: import('../types/api').PlanetSummary[] }>(
    'GET', `/systems/${system_id}/planets`
  );

// ── Types for API responses not in api.ts ─────────────────────
export interface UnitDefDto {
  id:       string;
  base_hp:  number;
  requires: string;
  cost:     { wood: number; stone: number; food: number; iron: number };
}

export interface TechDto {
  tech_id:            string;
  name:               string;
  description:        string;
  current_level:      number;
  max_level:          number;
  research_done_tick: number | null;
  eta_ticks:          number | null;
  ticks_per_level:    number;
  cost_per_level:     { wood: number; stone: number; food: number; iron: number };
}

export interface BattleReportDto {
  id:          number;
  battle_id:   number;
  attacker_id: number;
  defender_id: number;
  outcome:     string;
  attacker_units_snapshot: unknown;
  defender_units_snapshot: unknown;
  resources_looted:        unknown | null;
  started_tick: number;
  ended_tick:   number;
  read:         boolean;
}

export interface EmpireDto {
  id:         number;
  name:       string;
  created_by: number;
  members:    { player_id: number; username: string; role: string }[];
}
