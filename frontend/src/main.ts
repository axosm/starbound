/**
 * Starbound — main entry point.
 *
 * Views:
 *   planet  → Civ 6 hex tile surface
 *   system  → Stellaris solar system (orbit rings, planets)
 *   galaxy  → Instanced star field
 *
 * UI panels (toggle via toolbar):
 *   🔬 Research | 🛡 Empire | ⚔️ Battle reports
 *   Tile panel (click on tile) | Recruit panel (tile with barracks)
 */

import * as THREE from 'three';
import { gameInit, getSpeed, listUnits, setToken } from './net/api';
import { sseClient } from './net/sse';
import { clock, pollEvery } from './utils/timer';
import { createRenderer, handleResize } from './engine/scene/renderer';
import { OrbitCamera } from './engine/scene/camera_controls';
import { HexPlanet } from './engine/tiles/hex_planet';
import { SolarSystemView } from './engine/space/solar_system';
import { GalaxyView, addStarField } from './engine/space/galaxy_view';
import { mountAuthScreen } from './ui/screens/auth_screen';
import { Hud } from './ui/components/hud';
import { TilePanel } from './ui/components/tile_panel';
import { ResearchPanel } from './ui/components/research_panel';
import { EmpirePanel } from './ui/components/empire_panel';
import { RecruitPanel } from './ui/components/recruit_panel';
import { BattleReportsPanel } from './ui/components/battle_reports_panel';
import { getPlanet } from './net/api';
import type { GameInitResponse, PlanetSummary, SseBattleEvent } from './types/api';
import { Minimap } from './ui/components/minimap';
import { listSystemPlanets } from './net/api';

// ─── Bootstrap ───────────────────────────────────────────────────────────────

const app = document.getElementById('app')!;

const savedToken = localStorage.getItem('sb_token');
if (savedToken) setToken(savedToken);

mountAuthScreen(app, async (loginResp) => {
  localStorage.setItem('sb_token', loginResp.token);
  await startGame(loginResp.player_id, loginResp.username);
});

// ─── Game ────────────────────────────────────────────────────────────────────

async function startGame(playerId: number, _username: string) {
  app.querySelector('#auth-overlay')?.remove();

  // ── Canvas ──────────────────────────────────────────────────
  const canvas = document.createElement('canvas');
  canvas.id = 'game-canvas';
  canvas.style.cssText = 'position:fixed;inset:0;width:100%;height:100%;display:block;';
  app.appendChild(canvas);

  // ── Three.js core ───────────────────────────────────────────
  const renderer = createRenderer({ canvas });
  const scene    = new THREE.Scene();
  scene.background = new THREE.Color(0x020408);
  scene.fog        = new THREE.FogExp2(0x020408, 0.005);

  const ambient = new THREE.AmbientLight(0x334466, 0.5);
  scene.add(ambient);
  const sun = new THREE.DirectionalLight(0xfff5cc, 1.8);
  sun.position.set(20, 30, 10);
  sun.castShadow = true;
  sun.shadow.mapSize.set(2048, 2048);
  scene.add(sun);

  addStarField(scene, 10_000);

  const orbitCam = new OrbitCamera(canvas);

  // ── Game view systems ───────────────────────────────────────
  const hexPlanet  = new HexPlanet(scene);
  const solarView  = new SolarSystemView(scene);
  const galaxyView = new GalaxyView(scene);

  // ── UI ──────────────────────────────────────────────────────
  const hud           = new Hud(app);
  const tilePanel     = new TilePanel(app);
  const researchPanel = new ResearchPanel(app);
  const empirePanel   = new EmpirePanel(app, playerId);
  const recruitPanel  = new RecruitPanel(app);
  const reportsPanel  = new BattleReportsPanel(app, playerId);
  const minimap       = new Minimap(app);

  // ── Load game state ─────────────────────────────────────────
  let gameState: GameInitResponse;
  try {
    gameState = await gameInit();
  } catch (e) {
    alert(`Failed to load game: ${(e as Error).message}`);
    return;
  }

  clock.init(gameState.tick, gameState.speed, gameState.game_tick_ms);
  sseClient.connect();

  const speedInfo = await getSpeed();
  hud.init(gameState.tick, gameState.speed, speedInfo.allow_player_speed_change);

  // SSE battle alert → reports badge
  sseClient.on('battle_started', (d) => {
    const b = d as SseBattleEvent;
    reportsPanel.markUnread(1);
    hud.showAlert(`⚔️ Battle started at (${b.u},${b.v})!`, 'warn');
  });
  sseClient.on('battle_ended', () => {
    reportsPanel.markUnread(1);
  });

  // ── Toolbar ─────────────────────────────────────────────────
  buildToolbar(app, reportsPanel, researchPanel, empirePanel);

  // ── View management ─────────────────────────────────────────
  type ViewMode = 'planet' | 'system' | 'galaxy';
  let viewMode: ViewMode = 'planet';
  let currentPlanetId: number | null = gameState.home_planet?.id ?? null;
  let currentPlanets: PlanetSummary[] = gameState.home_planet ? [gameState.home_planet] : [];

  async function showPlanetView(planetId: number) {
    viewMode = 'planet';
    currentPlanetId = planetId;
    solarView.clear();
    galaxyView.clear();
    orbitCam.setRadius(15);
    tilePanel.hide();
    recruitPanel.hide();
    updateViewBtn();
    try {
      const pv = await getPlanet(planetId);
      hexPlanet.buildTiles(pv.tiles, pv.subdivision);
      minimap.drawPlanetView(pv.tiles, playerId);
    } catch (e) {
      hud.showAlert(`Planet load failed: ${(e as Error).message}`, 'error');
    }
  }

  async function showSystemView(systemId?: number) {
    viewMode = 'system';
    hexPlanet.clear();
    tilePanel.hide();
    recruitPanel.hide();
    minimap.clear();
    orbitCam.setRadius(40);
    // Load planets for the system from the API
    let planets = currentPlanets;
    if (systemId && systemId !== (currentPlanets[0]?.star_system_id ?? 0)) {
      try {
        const resp = await listSystemPlanets(systemId);
        planets = resp.planets;
        currentPlanets = planets;
      } catch { /* use existing */ }
    }
    solarView.build(planets);
    updateViewBtn();
  }

  function showGalaxyView() {
    viewMode = 'galaxy';
    hexPlanet.clear();
    solarView.clear();
    tilePanel.hide();
    recruitPanel.hide();
    orbitCam.setRadius(200);
    // Render current systems as a tiny cluster
    const systems = currentPlanets.map(p => ({
      id: p.star_system_id, x: p.x, y: 0, z: p.y, seed: p.seed,
    }));
    galaxyView.build(systems);
    updateViewBtn();
  }

  // ── View toggle button ──────────────────────────────────────
  const viewBtn = document.createElement('button');
  viewBtn.id = 'view-btn';
  viewBtn.style.cssText = `
    position:fixed; bottom:1rem; left:50%; transform:translateX(-50%);
    background:#0f172a; border:1px solid #1e3a5f; color:#38bdf8;
    padding:0.45rem 1.4rem; border-radius:8px; cursor:pointer;
    font-size:0.88rem; z-index:200; display:flex; gap:0.5rem;
  `;
  app.appendChild(viewBtn);

  function updateViewBtn() {
    if (viewMode === 'planet')  viewBtn.textContent = '🚀 Solar System';
    if (viewMode === 'system')  viewBtn.textContent = '🌌 Galaxy Map';
    if (viewMode === 'galaxy')  viewBtn.textContent = '🌍 Planet View';
  }
  updateViewBtn();

  viewBtn.addEventListener('click', () => {
    if (viewMode === 'planet')       showSystemView(currentPlanets);
    else if (viewMode === 'system')  showGalaxyView();
    else if (viewMode === 'galaxy' && currentPlanetId) showPlanetView(currentPlanetId);
  });

  // ── Raycasting / click handling ─────────────────────────────
  const raycaster = new THREE.Raycaster();
  const mouse     = new THREE.Vector2();

  canvas.addEventListener('click', (e) => {
    if (e.target !== canvas) return;
    const rect = canvas.getBoundingClientRect();
    mouse.x =  ((e.clientX - rect.left)  / rect.width)  * 2 - 1;
    mouse.y = -((e.clientY - rect.top)   / rect.height) * 2 + 1;
    raycaster.setFromCamera(mouse, orbitCam.camera);

    if (viewMode === 'planet') {
      const tile = hexPlanet.pick(raycaster);
      if (tile) {
        hexPlanet.clearHighlights();
        hexPlanet.highlight(tile.id, 0x38bdf8);
        tilePanel.show(tile, currentPlanetId!, playerId);
        // If tile has a barracks/space_dock show recruit
        if (tile.building && ['barracks','space_dock'].includes(tile.building.building_type)
            && !tile.building.construction_done_tick) {
          recruitPanel.show(tile, currentPlanetId!);
        } else {
          recruitPanel.hide();
        }
      }
    } else if (viewMode === 'system') {
      const planet = solarView.pick(raycaster);
      if (planet) showPlanetView(planet.id);
    }
  });

  // ── Unit polling (after ETA timers expire) ──────────────────
  pollEvery(8_000, async () => {
    try {
      const units = await listUnits();
      hud.showUnitEtas(units);
      if (viewMode === 'system') solarView.updateUnits(units);
    } catch { /* ignore */ }
  });

  // ── Render loop ─────────────────────────────────────────────
  const frameClock = new THREE.Clock();
  function animate() {
    requestAnimationFrame(animate);
    const delta = frameClock.getDelta();
    handleResize(renderer, orbitCam.camera);
    if (viewMode === 'planet') hexPlanet.update();
    if (viewMode === 'system') solarView.update();
    renderer.render(scene, orbitCam.camera);
  }
  animate();

  // ── Start ───────────────────────────────────────────────────
  if (currentPlanetId) await showPlanetView(currentPlanetId);
}

// ── Toolbar ──────────────────────────────────────────────────────────────────

function buildToolbar(
  container:      HTMLElement,
  reportsPanel:   BattleReportsPanel,
  researchPanel:  ResearchPanel,
  empirePanel:    EmpirePanel,
) {
  const bar = document.createElement('div');
  bar.id = 'toolbar';
  bar.innerHTML = `
    <button id="tb-research">🔬 Research</button>
    <button id="tb-empire">🛡 Empire</button>
    <button id="tb-reports">⚔️ Reports<span id="reports-badge" style="display:none"></span></button>
  `;
  container.appendChild(bar);

  reportsPanel.setBadge(bar.querySelector<HTMLElement>('#reports-badge')!);

  bar.querySelector('#tb-research')!.addEventListener('click', () => researchPanel.toggle());
  bar.querySelector('#tb-empire')!.addEventListener('click',   () => empirePanel.toggle());
  bar.querySelector('#tb-reports')!.addEventListener('click',  () => reportsPanel.toggle());

  injectToolbarCSS();
}

function injectToolbarCSS() {
  if (document.getElementById('toolbar-css')) return;
  const s = document.createElement('style');
  s.id = 'toolbar-css';
  s.textContent = `
    #toolbar {
      position: fixed; top: 3.6rem; left: 50%;
      transform: translateX(-50%);
      display: flex; gap: 0.4rem;
      z-index: 150; pointer-events: all;
    }
    #toolbar button {
      padding: 0.3rem 0.9rem;
      background: rgba(15,23,42,0.9);
      border: 1px solid #1e3a5f; color: #94a3b8;
      border-radius: 6px; cursor: pointer; font-size: 0.8rem;
      transition: all 0.15s; position: relative;
    }
    #toolbar button:hover { background: #0ea5e9; color: #fff; border-color: #0ea5e9; }
    #reports-badge {
      position: absolute; top: -4px; right: -4px;
      background: #ef4444; color: #fff;
      font-size: 0.65rem; width: 16px; height: 16px;
      border-radius: 50%; line-height: 16px; text-align: center;
    }
  `;
  document.head.appendChild(s);
}
