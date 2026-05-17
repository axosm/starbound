import { getResources, getSpeed, setSpeed } from '../../net/api';
import { sseClient } from '../../net/sse';
import { clock, formatSeconds } from '../../utils/timer';
import type { ResourcesResponse, SseBattleEvent, SseSpeedEvent, UnitDto } from '../../types/api';

export class Hud {
  private el:          HTMLElement;
  private resInterval: ReturnType<typeof setInterval> | null = null;

  constructor(private container: HTMLElement) {
    this.el = document.createElement('div');
    this.el.id = 'hud';
    this.container.appendChild(this.el);
    injectHudCSS();
    this.render();
    this.bindSSE();
  }

  private render() {
    this.el.innerHTML = `
      <div id="hud-top">
        <div id="hud-resources"></div>
        <div id="hud-tick">
          Tick <span id="tick-num">—</span>
          <span id="speed-badge">x1</span>
        </div>
      </div>
      <div id="hud-speed-controls"></div>
      <div id="hud-alerts" aria-live="polite"></div>
    `;
  }

  /** Call once after game init returns speed/tick info. */
  init(tick: number, speed: number, allowSpeedChange: boolean) {
    this.updateTickDisplay(tick, speed);
    if (allowSpeedChange) this.renderSpeedControls();
    this.startResourcePoll();
  }

  private updateTickDisplay(tick: number, speed: number) {
    const el = document.getElementById('tick-num');
    const badge = document.getElementById('speed-badge');
    if (el) el.textContent = String(tick);
    if (badge) {
      badge.textContent = `x${speed}`;
      badge.className = speed > 1 ? 'fast' : '';
    }
  }

  private renderSpeedControls() {
    const wrap = document.getElementById('hud-speed-controls')!;
    wrap.innerHTML = `
      <span class="speed-label">Speed:</span>
      ${[1, 2, 5, 10, 100].map(s =>
        `<button class="speed-btn" data-speed="${s}">x${s}</button>`
      ).join('')}
    `;
    wrap.querySelectorAll<HTMLButtonElement>('.speed-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        const s = Number(btn.dataset.speed);
        try {
          await setSpeed(s);
          // SSE will broadcast the change back
        } catch (e: unknown) {
          this.showAlert(`Speed change failed: ${(e as Error).message}`, 'error');
        }
      });
    });
  }

  private startResourcePoll() {
    const refresh = async () => {
      try {
        const r = await getResources();
        this.updateResources(r);
      } catch { /* ignore */ }
    };
    refresh();
    // Poll every 5 real seconds
    this.resInterval = setInterval(refresh, 5_000);

    // Also refresh whenever a tick fires
    sseClient.on('tick', (d: unknown) => {
      const { tick, speed } = d as { tick: number; speed: number };
      this.updateTickDisplay(tick, speed);
      refresh();
    });
  }

  private updateResources(r: ResourcesResponse) {
    const el = document.getElementById('hud-resources');
    if (!el) return;
    const visible = r.resources.filter(x => x.amount > 0 || x.rate_per_tick > 0);
    el.innerHTML = visible.map(res => `
      <div class="res-chip" title="${res.resource_type}">
        <span class="res-icon">${resIcon(res.resource_type)}</span>
        <span class="res-amount">${Math.floor(res.amount)}</span>
        <span class="res-rate">+${res.rate_per_tick.toFixed(1)}/t</span>
      </div>
    `).join('');
  }

  /** Show a battle alert. */
  private bindSSE() {
    sseClient.on('battle_started', (d: unknown) => {
      const b = d as SseBattleEvent;
      this.showAlert(`⚔️ Battle at (${b.u},${b.v}) — enemy approaching!`, 'warn');
    });
    sseClient.on('battle_ended', (d: unknown) => {
      const ev = d as { battle_id: number; outcome: string };
      this.showAlert(`🏳 Battle #${ev.battle_id} ended: ${ev.outcome}`, 'info');
    });
    sseClient.on('speed_changed', (d: unknown) => {
      const { speed, current_tick } = d as SseSpeedEvent;
      this.updateTickDisplay(current_tick, speed);
    });
  }

  showAlert(msg: string, level: 'info' | 'warn' | 'error' = 'info') {
    const el = document.getElementById('hud-alerts');
    if (!el) return;
    const div = document.createElement('div');
    div.className = `alert ${level}`;
    div.textContent = msg;
    el.prepend(div);
    setTimeout(() => div.remove(), 6000);
  }

  /** Update unit countdown badges (called from game loop when polling returns). */
  showUnitEtas(units: UnitDto[]) {
    // Attach countdown timer to units that are in transit
    const moving = units.filter(u => u.move_order);
    // Could render a side panel; for now just log
    if (moving.length) {
      const next = moving[0];
      const eta  = formatSeconds(clock.etaSeconds(next.move_order!.arrival_tick));
      console.debug(`Unit ${next.id} arriving in ${eta}`);
    }
  }

  destroy() {
    if (this.resInterval) clearInterval(this.resInterval);
    this.el.remove();
  }
}

function resIcon(rt: string): string {
  const map: Record<string, string> = {
    wood: '🪵', stone: '🪨', food: '🌾', water: '💧',
    coal: '⛏️', iron: '⚙️', petrol: '🛢️', copper: '🔩',
    silicon: '💎', uranium: '☢️', rare_earths: '✨', electricity: '⚡',
    deuterium: '🔮', dark_matter: '🌑', titanium: '🔘', antimatter: '💥',
  };
  return map[rt] ?? '📦';
}

function injectHudCSS() {
  if (document.getElementById('hud-css')) return;
  const s = document.createElement('style');
  s.id = 'hud-css';
  s.textContent = `
    #hud {
      position: fixed; top: 0; left: 0; right: 0;
      pointer-events: none; z-index: 100;
      font-size: 0.8rem;
    }
    #hud-top {
      display: flex; justify-content: space-between; align-items: center;
      padding: 0.5rem 1rem;
      background: linear-gradient(to bottom, rgba(0,0,0,0.8), transparent);
      pointer-events: all;
    }
    #hud-resources { display: flex; gap: 0.5rem; flex-wrap: wrap; }
    .res-chip {
      display: flex; align-items: center; gap: 3px;
      background: rgba(15,23,42,0.85);
      border: 1px solid #1e3a5f;
      border-radius: 4px; padding: 2px 6px;
    }
    .res-amount { color: #e2e8f0; font-weight: 600; }
    .res-rate   { color: #4ade80; font-size: 0.7rem; }
    #hud-tick {
      color: #94a3b8; font-family: monospace;
      display: flex; align-items: center; gap: 0.5rem;
    }
    #speed-badge {
      background: #1e3a5f; color: #38bdf8;
      border-radius: 4px; padding: 1px 6px; font-weight: 700;
    }
    #speed-badge.fast { background: #7c2d12; color: #fb923c; }
    #hud-speed-controls {
      display: flex; align-items: center; gap: 0.4rem;
      padding: 0.3rem 1rem;
      pointer-events: all;
    }
    .speed-label { color: #64748b; font-size: 0.75rem; }
    .speed-btn {
      padding: 2px 10px;
      background: #1e293b; border: 1px solid #334155;
      color: #94a3b8; border-radius: 4px; cursor: pointer;
      font-size: 0.78rem; transition: all 0.15s;
    }
    .speed-btn:hover { background: #0ea5e9; color: #fff; border-color: #0ea5e9; }
    #hud-alerts {
      position: fixed; bottom: 1rem; right: 1rem;
      display: flex; flex-direction: column; gap: 0.4rem;
      pointer-events: none;
    }
    .alert {
      padding: 0.5rem 1rem;
      border-radius: 6px; font-size: 0.85rem;
      animation: slideIn 0.3s ease;
      pointer-events: all;
    }
    .alert.info  { background: #0f172a; border: 1px solid #1e3a5f; color: #94a3b8; }
    .alert.warn  { background: #431407; border: 1px solid #c2410c; color: #fdba74; }
    .alert.error { background: #450a0a; border: 1px solid #dc2626; color: #fca5a5; }
    @keyframes slideIn {
      from { opacity: 0; transform: translateX(20px); }
      to   { opacity: 1; transform: translateX(0); }
    }
  `;
  document.head.appendChild(s);
}
