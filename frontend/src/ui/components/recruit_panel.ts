import { getUnitDefs, recruitUnit } from '../../net/api';
import type { UnitDefDto } from '../../net/api';
import type { TileDto } from '../../types/api';

export class RecruitPanel {
  private el:      HTMLElement;
  private visible  = false;
  private tile:    TileDto | null = null;
  private planetId = 0;

  constructor(private container: HTMLElement) {
    this.el = document.createElement('div');
    this.el.id = 'recruit-panel';
    this.el.style.display = 'none';
    this.container.appendChild(this.el);
    injectCSS();
  }

  async show(tile: TileDto, planetId: number) {
    this.tile     = tile;
    this.planetId = planetId;
    this.visible  = true;
    this.el.style.display = 'block';
    await this.refresh();
  }

  hide() { this.visible = false; this.el.style.display = 'none'; }

  private async refresh() {
    this.el.innerHTML = `
      <div class="rec-title">⚔️ Recruit Units<button id="rec-close">✕</button></div>
      <div id="rec-list">Loading…</div>
    `;
    this.el.querySelector('#rec-close')!.addEventListener('click', () => this.hide());

    try {
      const { unit_defs } = await getUnitDefs();
      this.renderDefs(unit_defs);
    } catch (e: unknown) {
      (this.el.querySelector('#rec-list') as HTMLElement).textContent =
        `Error: ${(e as Error).message}`;
    }
  }

  private renderDefs(defs: UnitDefDto[]) {
    const list = this.el.querySelector('#rec-list')!;
    list.innerHTML = defs.map(d => `
      <div class="rec-row" data-id="${d.id}">
        <div class="rec-info">
          <span class="rec-name">${d.id.replace(/_/g,' ')}</span>
          <span class="rec-hp">HP: ${d.base_hp}</span>
          <span class="rec-req">Needs: ${d.requires.replace(/_/g,' ')}</span>
          <span class="rec-cost">
            ${d.cost.wood  ? `🪵${d.cost.wood}` : ''}
            ${d.cost.stone ? `🪨${d.cost.stone}` : ''}
            ${d.cost.food  ? `🌾${d.cost.food}` : ''}
            ${d.cost.iron  ? `⚙️${d.cost.iron}` : ''}
          </span>
        </div>
        <div class="rec-right">
          <input type="number" class="rec-count" value="1" min="1" max="500" />
          <button class="rec-btn" data-id="${d.id}">Recruit</button>
        </div>
      </div>
    `).join('');

    list.querySelectorAll<HTMLButtonElement>('.rec-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        if (!this.tile) return;
        const row   = btn.closest<HTMLElement>('.rec-row')!;
        const count = parseInt((row.querySelector<HTMLInputElement>('.rec-count')!).value, 10);
        const type  = btn.dataset.id!;
        btn.disabled = true;
        try {
          await recruitUnit({
            unit_type: type, count,
            planet_id: this.planetId,
            face: this.tile!.face,
            u:    this.tile!.u,
            v:    this.tile!.v,
          });
          btn.textContent = '✓ Done';
        } catch (e: unknown) {
          alert((e as Error).message);
          btn.disabled = false;
        }
      });
    });
  }

  destroy() { this.el.remove(); }
}

function injectCSS() {
  if (document.getElementById('rec-css')) return;
  const s = document.createElement('style');
  s.id = 'rec-css';
  s.textContent = `
    #recruit-panel {
      position: fixed; right: 290px; top: 5rem;
      width: 290px; max-height: 75vh; overflow-y: auto;
      background: rgba(10,15,30,0.97);
      border: 1px solid #1e3a5f; border-radius: 10px;
      padding: 1rem; z-index: 200; color: #e2e8f0; font-size: 0.82rem;
    }
    .rec-title {
      font-size: 1rem; font-weight: 700; color: #f97316;
      display: flex; justify-content: space-between; margin-bottom: 0.75rem;
    }
    #rec-close { background:none; border:none; color:#64748b; cursor:pointer; font-size:1rem; }
    .rec-row {
      display: flex; justify-content: space-between; align-items: center;
      padding: 0.45rem 0; border-bottom: 1px solid #1e293b;
    }
    .rec-name { font-weight:600; display:block; text-transform:capitalize; }
    .rec-hp   { color:#94a3b8; font-size:0.75rem; }
    .rec-req  { color:#64748b; font-size:0.72rem; display:block; text-transform:capitalize; }
    .rec-cost { color:#fbbf24; font-size:0.72rem; display:block; margin-top:1px; gap:4px; }
    .rec-right { display:flex; flex-direction:column; gap:3px; flex-shrink:0; margin-left:0.5rem; }
    .rec-count {
      width: 56px; padding: 2px 4px; text-align: center;
      background:#1e293b; border:1px solid #334155; border-radius:3px; color:#e2e8f0;
    }
    .rec-btn {
      padding: 3px 8px; background:#c2410c; color:#fff;
      border:none; border-radius:4px; cursor:pointer; font-size:0.75rem;
    }
    .rec-btn:disabled { opacity:0.5; }
  `;
  document.head.appendChild(s);
}
