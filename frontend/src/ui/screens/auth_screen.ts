import { login, register, setToken } from '../../net/api';
import type { LoginResponse } from '../../types/api';

export function mountAuthScreen(
  container: HTMLElement,
  onSuccess:  (resp: LoginResponse) => void,
) {
  container.innerHTML = `
    <div id="auth-overlay">
      <div id="auth-box">
        <h1>⭐ STARBOUND</h1>
        <p class="sub">A galaxy awaits</p>
        <div id="auth-tabs">
          <button class="tab active" data-tab="login">Login</button>
          <button class="tab" data-tab="register">Register</button>
        </div>
        <div id="auth-form-login" class="auth-form">
          <input id="login-email"    type="email"    placeholder="Email" />
          <input id="login-password" type="password" placeholder="Password" />
          <button id="btn-login">Login</button>
        </div>
        <div id="auth-form-register" class="auth-form" style="display:none">
          <input id="reg-username" type="text"     placeholder="Username" />
          <input id="reg-email"    type="email"    placeholder="Email" />
          <input id="reg-password" type="password" placeholder="Password (min 8)" />
          <button id="btn-register">Create Account</button>
        </div>
        <p id="auth-error" class="error"></p>
      </div>
    </div>
  `;

  injectAuthCSS();

  // Tab switching
  container.querySelectorAll<HTMLButtonElement>('.tab').forEach(btn => {
    btn.addEventListener('click', () => {
      container.querySelectorAll('.tab').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const tab = btn.dataset.tab!;
      (container.querySelector('#auth-form-login') as HTMLElement).style.display =
        tab === 'login' ? 'flex' : 'none';
      (container.querySelector('#auth-form-register') as HTMLElement).style.display =
        tab === 'register' ? 'flex' : 'none';
      (container.querySelector('#auth-error') as HTMLElement).textContent = '';
    });
  });

  const errEl = container.querySelector<HTMLElement>('#auth-error')!;

  // Login
  container.querySelector('#btn-login')!.addEventListener('click', async () => {
    const email    = (container.querySelector<HTMLInputElement>('#login-email')!).value;
    const password = (container.querySelector<HTMLInputElement>('#login-password')!).value;
    errEl.textContent = '';
    try {
      const resp = await login(email, password);
      setToken(resp.token);
      onSuccess(resp);
    } catch (e: unknown) {
      errEl.textContent = (e as Error).message;
    }
  });

  // Register
  container.querySelector('#btn-register')!.addEventListener('click', async () => {
    const username = (container.querySelector<HTMLInputElement>('#reg-username')!).value;
    const email    = (container.querySelector<HTMLInputElement>('#reg-email')!).value;
    const password = (container.querySelector<HTMLInputElement>('#reg-password')!).value;
    errEl.textContent = '';
    try {
      const resp = await register(username, email, password);
      setToken(resp.token);
      onSuccess(resp);
    } catch (e: unknown) {
      errEl.textContent = (e as Error).message;
    }
  });
}

function injectAuthCSS() {
  if (document.getElementById('auth-css')) return;
  const style = document.createElement('style');
  style.id = 'auth-css';
  style.textContent = `
    #auth-overlay {
      position: fixed; inset: 0;
      background: radial-gradient(ellipse at center, #0a0e1a 0%, #000 100%);
      display: flex; align-items: center; justify-content: center;
      z-index: 1000;
    }
    #auth-box {
      background: #0f172a;
      border: 1px solid #1e3a5f;
      border-radius: 12px;
      padding: 2.5rem 3rem;
      width: 360px;
      text-align: center;
      box-shadow: 0 0 40px rgba(56,189,248,0.15);
    }
    #auth-box h1 { font-size: 2rem; color: #38bdf8; letter-spacing: 4px; margin-bottom: 0.3rem; }
    #auth-box .sub { color: #64748b; font-size: 0.85rem; margin-bottom: 1.5rem; }
    #auth-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.2rem; }
    #auth-tabs .tab {
      flex: 1; padding: 0.5rem;
      background: #1e293b; border: 1px solid #334155;
      color: #94a3b8; border-radius: 6px; cursor: pointer; font-size: 0.9rem;
      transition: all 0.2s;
    }
    #auth-tabs .tab.active { background: #0ea5e9; color: #fff; border-color: #0ea5e9; }
    .auth-form { display: flex; flex-direction: column; gap: 0.75rem; }
    .auth-form input {
      padding: 0.65rem 1rem;
      background: #1e293b; border: 1px solid #334155;
      border-radius: 6px; color: #e2e8f0; font-size: 0.95rem;
      outline: none; transition: border-color 0.2s;
    }
    .auth-form input:focus { border-color: #38bdf8; }
    .auth-form button {
      margin-top: 0.5rem; padding: 0.7rem;
      background: #0284c7; color: #fff;
      border: none; border-radius: 6px;
      font-size: 1rem; cursor: pointer; font-weight: 600;
      transition: background 0.2s;
    }
    .auth-form button:hover { background: #0369a1; }
    .error { color: #f87171; font-size: 0.85rem; margin-top: 0.75rem; min-height: 1.2em; }
  `;
  document.head.appendChild(style);
}
