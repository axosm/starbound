import { getResearch, startResearch } from '../../net/api';
import type { TechDto } from '../../net/api';
import { clock } from '../../utils/timer';

export class ResearchPanel {
  private el: HTMLElement;
  private visible = false;

  constructor(private container: HTMLElement) {
    this.el = document.createElement('div');
    this.el.id = 'research-panel';
    this.el.style.display = 'none';
    this.container.appendChild(this.el);
    injectCSS();
  }

  async toggle() {
    this.visible = !this.visible;
    if (this.visible) { await this.refresh(); this.el.style.display = 'block'; }
    else              { this.el.style.display = 'none'; }
  }

  async refresh() {
    this.el.innerHTML = '<div class="rp-title">🔬 Research<button id="rp-close">✕</button></div><div id="rp-list">Loading…</div>';
    this.el.querySelector('#rp-close')!.addEventListener('click', () => {
      this.visible = false; this.el.style.display = 'none';
    });
    try {
      const { techs } = await getResearch();
      this.renderTechs(techs);
    } catch (e: unknown) {
      (this.el.querySelector('#rp-list') as HTMLElement).textContent =
        `Error: ${(e as Error).message}`;
    }
  }

  private renderTechs(techs: TechDto[]) {
    const list = this.el.querySelector('#rp-list')!;
    list.innerHTML = techs.map(t => {
      const inProgress = t.research_done_tick !== null;
      const maxed = t.current_level >= t.max_level;
      const eta = inProgress ? clock.formatEta(t.research_done_tick!) : '';
      return `
        <div class="tech-row ${inProgress ? 'in-progress' : ''} ${maxed ? 'maxed' : ''}">
          <div class="tech-info">
            <span class="tech-name">${t.name}</span>
            <span class="tech-level">Lv ${t.current_level}/${t.max_level}</span>
            <span class="tech-desc">${t.description}</span>
          </div>
          <div class="tech-right">
            ${inProgress
              ? `<span class="eta">⏳ ${eta}</span>`
              : maxed
              ? `<span class="maxed-badge">MAX</span>`
              : `<button class="research-btn" data-id="${t.tech_id}">
                   Research (${t.ticks_per_level}t)
                 </button>`
            }
          </div>
        </div>`;
    }).join('');

    list.querySelectorAll<HTMLButtonElement>('.research-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        btn.disabled = true;
        btn.textContent = '…';
        try {
          await startResearch(btn.dataset.id!);
          await this.refresh();
        } catch (e: unknown) {
          alert((e as Error).message);
          btn.disabled = false;
          btn.textContent = 'Research';
        }
      });
    });
  }

  destroy() { this.el.remove(); }
}

function injectCSS() {
  if (document.getElementById('rp-css')) return;
  const s = document.createElement('style');
  s.id = 'rp-css';
  s.textContent = `
    #research-panel {
      position: fixed; left: 1rem; top: 5rem;
      width: 340px; max-height: 75vh; overflow-y: auto;
      background: rgba(10,15,30,0.97);
      border: 1px solid #1e3a5f; border-radius: 10px;
      padding: 1rem; z-index: 200; color: #e2e8f0; font-size: 0.82rem;
    }
    .rp-title {
      font-size: 1rem; font-weight: 700; color: #38bdf8;
      display: flex; justify-content: space-between; margin-bottom: 0.75rem;
    }
    #rp-close {
      background: none; border: none; color: #64748b; cursor: pointer; font-size: 1rem;
    }
    .tech-row {
      display: flex; justify-content: space-between; align-items: center;
      padding: 0.5rem 0; border-bottom: 1px solid #1e293b;
    }
    .tech-name  { font-weight: 600; color: #e2e8f0; display: block; }
    .tech-level { color: #38bdf8; font-size: 0.75rem; }
    .tech-desc  { color: #64748b; font-size: 0.72rem; display: block; margin-top: 2px; }
    .tech-right { flex-shrink: 0; margin-left: 0.5rem; }
    .research-btn {
      padding: 3px 8px; background: #0284c7; color: #fff;
      border: none; border-radius: 4px; cursor: pointer; font-size: 0.75rem;
      white-space: nowrap;
    }
    .research-btn:disabled { opacity: 0.5; }
    .eta         { color: #fb923c; font-size: 0.78rem; }
    .maxed-badge { color: #4ade80; font-size: 0.75rem; font-weight: 700; }
    .in-progress .tech-name::after { content: ' 🔬'; }
    .maxed .tech-name { color: #4ade80; }
  `;
  document.head.appendChild(s);
}
