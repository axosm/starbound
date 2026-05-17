import { buildOnTile } from '../../net/api';
import { clock } from '../../utils/timer';
import type { TileDto } from '../../types/api';

const BUILDING_LIST = [
  'town_center','lumber_mill','quarry','farm','water_pump',
  'mine','barracks','storage','wall','watchtower',
  'lab','launch_pad','space_dock','stargate',
];

export class TilePanel {
  private el:       HTMLElement;
  private planetId: number | null = null;

  constructor(private container: HTMLElement) {
    this.el = document.createElement('div');
    this.el.id = 'tile-panel';
    this.el.style.display = 'none';
    this.container.appendChild(this.el);
    injectPanelCSS();
  }

  show(tile: TileDto, planetId: number, myPlayerId: number) {
    this.planetId = planetId;
    const mine = tile.owner_player_id === myPlayerId;
    const bld  = tile.building;

    this.el.style.display = 'block';
    this.el.innerHTML = `
      <button id="tp-close">✕</button>
      <h3>${tile.tile_type.toUpperCase()} tile</h3>
      <div class="tp-stat">Yield: <strong>${(tile.yield_quality * 100).toFixed(0)}%</strong></div>
      ${tile.rare_deposit ? `<div class="tp-stat deposit">Deposit: <strong>${tile.rare_deposit}</strong></div>` : ''}
      <div class="tp-stat">Owner: <strong>${tile.owner_player_id ?? 'Unclaimed'}</strong></div>

      ${bld ? `
        <div class="tp-bld">
          <div>🏗 <strong>${bld.building_type}</strong> Lv${bld.level}</div>
          <div class="hp-bar"><div class="hp-fill" style="width:${(bld.hp / bld.max_hp * 100).toFixed(0)}%"></div></div>
          ${bld.construction_done_tick
            ? `<div class="building-in">Building... ${clock.formatEta(bld.construction_done_tick)}</div>`
            : ''}
          ${bld.under_attack ? `<div class="under-attack">⚔️ Under attack!</div>` : ''}
        </div>
      ` : ''}

      ${tile.units.length ? `
        <div class="tp-units">
          ${tile.units.map(u => `
            <div class="unit-row ${u.player_id === myPlayerId ? 'friendly' : 'enemy'}">
              ${u.unit_type} ×${u.count} HP:${u.hp}
            </div>
          `).join('')}
        </div>
      ` : ''}

      ${mine && !bld ? `
        <div class="tp-build">
          <p>Build:</p>
          <div id="build-list">
            ${BUILDING_LIST.map(b =>
              `<button class="build-btn" data-type="${b}">${b.replace('_',' ')}</button>`
            ).join('')}
          </div>
        </div>
      ` : ''}
    `;

    this.el.querySelector('#tp-close')!.addEventListener('click', () => this.hide());

    this.el.querySelectorAll<HTMLButtonElement>('.build-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        const type = btn.dataset.type!;
        try {
          await buildOnTile(planetId, tile.face, tile.u, tile.v, type);
          btn.textContent = '✓ Queued';
          btn.disabled = true;
        } catch (e: unknown) {
          alert((e as Error).message);
        }
      });
    });
  }

  hide() { this.el.style.display = 'none'; }
  destroy() { this.el.remove(); }
}

function injectPanelCSS() {
  if (document.getElementById('tp-css')) return;
  const s = document.createElement('style');
  s.id = 'tp-css';
  s.textContent = `
    #tile-panel {
      position: fixed; right: 1rem; top: 5rem;
      width: 260px;
      background: rgba(10,15,30,0.95);
      border: 1px solid #1e3a5f;
      border-radius: 10px;
      padding: 1rem;
      z-index: 200;
      color: #e2e8f0;
      font-size: 0.85rem;
    }
    #tile-panel h3 { margin-bottom: 0.6rem; color: #38bdf8; text-transform: capitalize; }
    #tp-close {
      position: absolute; top: 0.5rem; right: 0.5rem;
      background: none; border: none; color: #64748b; cursor: pointer; font-size: 1rem;
    }
    .tp-stat { margin: 0.25rem 0; color: #94a3b8; }
    .deposit strong { color: #fbbf24; }
    .tp-bld { margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid #1e3a5f; }
    .hp-bar { background: #1e293b; height: 6px; border-radius: 3px; margin: 4px 0; }
    .hp-fill { background: #22c55e; height: 100%; border-radius: 3px; transition: width 0.3s; }
    .building-in { color: #fb923c; font-size: 0.78rem; }
    .under-attack { color: #f87171; font-weight: 700; }
    .tp-units { margin-top: 0.75rem; }
    .unit-row { padding: 2px 6px; border-radius: 4px; margin: 2px 0; font-size: 0.8rem; }
    .unit-row.friendly { background: rgba(56,189,248,0.1); color: #7dd3fc; }
    .unit-row.enemy    { background: rgba(239,68,68,0.1);  color: #fca5a5; }
    .tp-build { margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid #1e3a5f; }
    .tp-build p { color: #64748b; margin-bottom: 0.4rem; font-size: 0.78rem; }
    #build-list { display: flex; flex-wrap: wrap; gap: 0.3rem; }
    .build-btn {
      padding: 3px 8px;
      background: #1e293b; border: 1px solid #334155;
      color: #94a3b8; border-radius: 4px; cursor: pointer; font-size: 0.75rem;
      text-transform: capitalize; transition: all 0.15s;
    }
    .build-btn:hover:not(:disabled) { background: #0284c7; color: #fff; border-color: #0284c7; }
    .build-btn:disabled { opacity: 0.5; cursor: default; }
  `;
  document.head.appendChild(s);
}
