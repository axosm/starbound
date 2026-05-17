import { getBattleReports, markReportsRead } from '../../net/api';
import type { BattleReportDto } from '../../net/api';

export class BattleReportsPanel {
  private el:      HTMLElement;
  private visible  = false;
  private unread   = 0;
  private badgeEl: HTMLElement | null = null;

  constructor(private container: HTMLElement, private playerId: number) {
    this.el = document.createElement('div');
    this.el.id = 'reports-panel';
    this.el.style.display = 'none';
    this.container.appendChild(this.el);
    injectCSS();
  }

  setBadge(el: HTMLElement) { this.badgeEl = el; }

  async toggle() {
    this.visible = !this.visible;
    if (this.visible) {
      await this.refresh();
      this.el.style.display = 'block';
      // Mark as read after opening
      markReportsRead().catch(() => {});
      this.unread = 0;
      this.updateBadge();
    } else {
      this.el.style.display = 'none';
    }
  }

  markUnread(n: number) {
    this.unread = n;
    this.updateBadge();
  }

  private updateBadge() {
    if (this.badgeEl) {
      this.badgeEl.textContent = this.unread > 0 ? String(this.unread) : '';
      this.badgeEl.style.display = this.unread > 0 ? 'inline' : 'none';
    }
  }

  async refresh() {
    this.el.innerHTML = `
      <div class="rpt-title">⚔️ Battle Reports<button id="rpt-close">✕</button></div>
      <div id="rpt-list">Loading…</div>
    `;
    this.el.querySelector('#rpt-close')!.addEventListener('click', () => {
      this.visible = false; this.el.style.display = 'none';
    });
    try {
      const { reports } = await getBattleReports(30);
      this.renderReports(reports);
    } catch (e: unknown) {
      (this.el.querySelector('#rpt-list') as HTMLElement).textContent =
        `Error: ${(e as Error).message}`;
    }
  }

  private renderReports(reports: BattleReportDto[]) {
    const list = this.el.querySelector('#rpt-list')!;
    if (!reports.length) {
      list.innerHTML = '<p class="rpt-empty">No battles yet.</p>';
      return;
    }

    const outcomeEmoji: Record<string, string> = {
      attacker_victory:  '🏆 Victory',
      defender_victory:  '🛡 Defended',
      attacker_retreated:'🏃 Retreated',
      defender_retreated:'🏃 Enemy retreated',
      attacker_looted:   '💰 Looted',
      draw:              '🤝 Draw',
    };

    list.innerHTML = reports.map(r => {
      const iAm     = r.attacker_id === this.playerId ? 'attacker' : 'defender';
      const outcome = outcomeEmoji[r.outcome] ?? r.outcome;
      const won     = (iAm === 'attacker' && r.outcome === 'attacker_victory') ||
                      (iAm === 'defender' && r.outcome === 'defender_victory');
      return `
        <div class="rpt-row ${r.read ? '' : 'unread'} ${won ? 'won' : 'lost'}">
          <div class="rpt-outcome">${outcome}</div>
          <div class="rpt-meta">
            ${iAm === 'attacker' ? '⚔️ You attacked' : '🛡 You defended'}
            · Tick ${r.ended_tick}
          </div>
          ${r.resources_looted
            ? `<div class="rpt-loot">Loot: ${JSON.stringify(r.resources_looted)}</div>`
            : ''}
        </div>`;
    }).join('');
  }

  destroy() { this.el.remove(); }
}

function injectCSS() {
  if (document.getElementById('rpt-css')) return;
  const s = document.createElement('style');
  s.id = 'rpt-css';
  s.textContent = `
    #reports-panel {
      position: fixed; left: 1rem; bottom: 3.5rem;
      width: 300px; max-height: 50vh; overflow-y: auto;
      background: rgba(10,15,30,0.97);
      border: 1px solid #1e3a5f; border-radius: 10px;
      padding: 1rem; z-index: 200; color: #e2e8f0; font-size: 0.82rem;
    }
    .rpt-title {
      font-size: 1rem; font-weight: 700; color: #ef4444;
      display: flex; justify-content: space-between; margin-bottom: 0.75rem;
    }
    #rpt-close { background:none; border:none; color:#64748b; cursor:pointer; font-size:1rem; }
    .rpt-empty { color: #64748b; }
    .rpt-row {
      padding: 0.45rem 0.6rem; border-radius: 6px; margin-bottom: 0.4rem;
      border-left: 3px solid #334155;
    }
    .rpt-row.unread { background: rgba(30,58,95,0.3); }
    .rpt-row.won    { border-left-color: #4ade80; }
    .rpt-row.lost   { border-left-color: #f87171; }
    .rpt-outcome { font-weight: 700; }
    .rpt-meta    { color: #64748b; font-size: 0.75rem; margin-top: 2px; }
    .rpt-loot    { color: #fbbf24; font-size: 0.72rem; margin-top: 2px; }
  `;
  document.head.appendChild(s);
}
