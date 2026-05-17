import { getToken } from './api';

type Handler = (data: unknown) => void;

export class SseClient {
  private es:       EventSource | null = null;
  private handlers: Map<string, Handler[]> = new Map();
  private lastId    = 0;
  private reconnectMs = 3000;

  connect() {
    const token = getToken();
    if (!token) return;
    this.es = new EventSource(`/api/events?token=${encodeURIComponent(token)}`);

    this.es.onopen = () => {
      console.log('[SSE] connected');
      this.reconnectMs = 3000;
      this.fetchMissed();
    };

    this.es.onerror = () => {
      console.warn('[SSE] disconnected — reconnecting in', this.reconnectMs, 'ms');
      this.es?.close();
      setTimeout(() => this.connect(), this.reconnectMs);
      this.reconnectMs = Math.min(this.reconnectMs * 2, 30_000);
    };

    // Tick
    this.es.addEventListener('tick', (e: MessageEvent) => {
      this.emit('tick', JSON.parse(e.data));
    });
    // Speed changed
    this.es.addEventListener('speed_changed', (e: MessageEvent) => {
      this.emit('speed_changed', JSON.parse(e.data));
    });
    // Battle
    this.es.addEventListener('battle_started', (e: MessageEvent) => {
      this.emit('battle_started', JSON.parse(e.data));
    });
    this.es.addEventListener('battle_ended', (e: MessageEvent) => {
      this.emit('battle_ended', JSON.parse(e.data));
    });
  }

  on(event: string, handler: Handler) {
    if (!this.handlers.has(event)) this.handlers.set(event, []);
    this.handlers.get(event)!.push(handler);
  }

  off(event: string, handler: Handler) {
    const arr = this.handlers.get(event);
    if (arr) this.handlers.set(event, arr.filter(h => h !== handler));
  }

  private emit(event: string, data: unknown) {
    this.handlers.get(event)?.forEach(h => h(data));
  }

  private async fetchMissed() {
    try {
      const token = getToken();
      if (!token) return;
      const res = await fetch(`/api/events/missed?since_id=${this.lastId}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) return;
      const { events } = await res.json();
      for (const ev of events) {
        if (ev.id > this.lastId) this.lastId = ev.id;
        this.emit(ev.event_type, JSON.parse(ev.payload));
      }
    } catch { /* ignore */ }
  }

  disconnect() { this.es?.close(); this.es = null; }
}

export const sseClient = new SseClient();
