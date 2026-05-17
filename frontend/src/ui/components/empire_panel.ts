import { createEmpire, getMyEmpire, inviteToEmpire, kickFromEmpire, leaveEmpire } from '../../net/api';
import type { EmpireDto } from '../../net/api';

export class EmpirePanel {
  private el:      HTMLElement;
  private visible  = false;

  constructor(private container: HTMLElement, private playerId: number) {
    this.el = document.createElement('div');
    this.el.id = 'empire-panel';
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
    this.el.innerHTML = '<div class="ep-title">🛡 Empire<button id="ep-close">✕</button></div><div id="ep-body">Loading…</div>';
    this.el.querySelector('#ep-close')!.addEventListener('click', () => {
      this.visible = false; this.el.style.display = 'none';
    });
    try {
      const { empire } = await getMyEmpire() as { empire: EmpireDto | null };
      if (empire) this.renderEmpire(empire);
      else        this.renderCreate();
    } catch { this.renderCreate(); }
  }

  private renderCreate() {
    const body = this.el.querySelector('#ep-body')!;
    body.innerHTML = `
      <p class="ep-none">You are not in an empire.</p>
      <input id="ep-name" type="text" placeholder="Empire name" />
      <button id="ep-create">Found Empire</button>
    `;
    this.el.querySelector('#ep-create')!.addEventListener('click', async () => {
      const name = (this.el.querySelector<HTMLInputElement>('#ep-name')!).value.trim();
      if (!name) return;
      try {
        await createEmpire(name);
        await this.refresh();
      } catch (e: unknown) { alert((e as Error).message); }
    });
  }

  private renderEmpire(e: EmpireDto) {
    const isLeader = e.members.find(m => m.player_id === this.playerId)?.role === 'leader';
    const body = this.el.querySelector('#ep-body')!;
    body.innerHTML = `
      <div class="ep-name-row">
        <strong>${e.name}</strong>
        <span class="ep-count">${e.members.length} members</span>
      </div>
      <div class="ep-members">
        ${e.members.map(m => `
          <div class="ep-member">
            <span>${m.username}</span>
            <span class="ep-role ${m.role}">${m.role}</span>
            ${isLeader && m.player_id !== this.playerId
              ? `<button class="kick-btn" data-pid="${m.player_id}" data-eid="${e.id}">Kick</button>`
              : ''}
          </div>
        `).join('')}
      </div>
      ${isLeader ? `
        <div class="ep-invite">
          <input id="ep-inv-name" type="text" placeholder="Username to invite" />
          <button id="ep-invite-btn" data-eid="${e.id}">Invite</button>
        </div>
      ` : ''}
      <button id="ep-leave" class="leave-btn">Leave Empire</button>
    `;

    body.querySelectorAll<HTMLButtonElement>('.kick-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        if (!confirm('Kick this player?')) return;
        await kickFromEmpire(Number(btn.dataset.eid), Number(btn.dataset.pid));
        await this.refresh();
      });
    });

    const invBtn = body.querySelector<HTMLButtonElement>('#ep-invite-btn');
    invBtn?.addEventListener('click', async () => {
      const name = (body.querySelector<HTMLInputElement>('#ep-inv-name')!).value.trim();
      if (!name) return;
      try {
        await inviteToEmpire(e.id, name);
        await this.refresh();
      } catch (ex: unknown) { alert((ex as Error).message); }
    });

    body.querySelector('#ep-leave')!.addEventListener('click', async () => {
      if (!confirm('Leave empire?')) return;
      await leaveEmpire();
      await this.refresh();
    });
  }

  destroy() { this.el.remove(); }
}

function injectCSS() {
  if (document.getElementById('ep-css')) return;
  const s = document.createElement('style');
  s.id = 'ep-css';
  s.textContent = `
    #empire-panel {
      position: fixed; right: 1rem; top: 5rem;
      width: 280px; max-height: 75vh; overflow-y: auto;
      background: rgba(10,15,30,0.97);
      border: 1px solid #1e3a5f; border-radius: 10px;
      padding: 1rem; z-index: 201; color: #e2e8f0; font-size: 0.82rem;
    }
    .ep-title {
      font-size: 1rem; font-weight: 700; color: #a78bfa;
      display: flex; justify-content: space-between; margin-bottom: 0.75rem;
    }
    #ep-close { background:none; border:none; color:#64748b; cursor:pointer; font-size:1rem; }
    .ep-none  { color: #64748b; margin-bottom: 0.75rem; }
    #ep-name, #ep-inv-name {
      width: 100%; padding: 0.4rem 0.6rem; margin-bottom: 0.4rem;
      background: #1e293b; border: 1px solid #334155; border-radius: 4px;
      color: #e2e8f0; font-size: 0.85rem;
    }
    #ep-create, #ep-invite-btn {
      width: 100%; padding: 0.4rem;
      background: #7c3aed; color: #fff; border: none; border-radius: 4px;
      cursor: pointer; font-size: 0.85rem; margin-bottom: 0.5rem;
    }
    .ep-name-row { display:flex; justify-content:space-between; margin-bottom:0.5rem; }
    .ep-count    { color: #64748b; font-size: 0.75rem; }
    .ep-member   { display:flex; align-items:center; gap:0.5rem; padding: 3px 0; }
    .ep-member span:first-child { flex:1; }
    .ep-role { font-size:0.7rem; padding:1px 5px; border-radius:3px; }
    .ep-role.leader  { background:#7c3aed; color:#fff; }
    .ep-role.officer { background:#0284c7; color:#fff; }
    .ep-role.member  { background:#334155; color:#94a3b8; }
    .kick-btn {
      padding:1px 6px; background:#7f1d1d; color:#fca5a5;
      border:none; border-radius:3px; cursor:pointer; font-size:0.72rem;
    }
    .ep-invite { margin: 0.6rem 0; }
    .leave-btn {
      width:100%; padding:0.35rem; margin-top:0.5rem;
      background:#1e293b; color:#f87171; border:1px solid #7f1d1d;
      border-radius:4px; cursor:pointer; font-size:0.8rem;
    }
  `;
  document.head.appendChild(s);
}
