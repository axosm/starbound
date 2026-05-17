import { sseClient } from '../net/sse';
import type { SseSpeedEvent, SseTickEvent } from '../types/api';

/**
 * TickClock — a client-side clock that stays in sync with the server.
 *
 * - Listens to SSE "tick" events and resyncs when drift detected.
 * - Adjusts countdown displays when speed changes.
 */
export class TickClock {
  private tick         = 0;
  private speed        = 1;
  private gameTickMs   = 60_000;
  private realTickMs   = 60_000;
  private lastTickAt   = Date.now();
  private listeners    = new Set<() => void>();

  init(tick: number, speed: number, gameTickMs: number) {
    this.tick       = tick;
    this.speed      = speed;
    this.gameTickMs = gameTickMs;
    this.realTickMs = gameTickMs / speed;
    this.lastTickAt = Date.now();

    sseClient.on('tick', (d) => {
      const { tick, speed, real_tick_ms } = d as SseTickEvent;
      this.tick       = tick;
      this.speed      = speed;
      this.realTickMs = real_tick_ms;
      this.lastTickAt = Date.now();
      this.notify();
    });

    sseClient.on('speed_changed', (d) => {
      const { speed, real_tick_ms, current_tick } = d as SseSpeedEvent;
      this.speed      = speed;
      this.realTickMs = real_tick_ms;
      this.tick       = current_tick;
      this.lastTickAt = Date.now();
      this.notify();
    });
  }

  /** Estimated current game tick (interpolated). */
  estimatedTick(): number {
    const elapsed = Date.now() - this.lastTickAt;
    return this.tick + elapsed / this.realTickMs;
  }

  /** Estimated real seconds until a future game tick. */
  etaSeconds(arrivalTick: number): number {
    const ticksLeft = arrivalTick - this.estimatedTick();
    return Math.max(0, ticksLeft * this.realTickMs / 1000);
  }

  /** Format ETA as "HH:MM:SS" or "Mm Ss". */
  formatEta(arrivalTick: number): string {
    return formatSeconds(this.etaSeconds(arrivalTick));
  }

  onChange(fn: () => void) { this.listeners.add(fn); }
  offChange(fn: () => void) { this.listeners.delete(fn); }
  private notify() { this.listeners.forEach(fn => fn()); }

  getSpeed()      { return this.speed; }
  getRealTickMs() { return this.realTickMs; }
  getCurrentTick(){ return this.tick; }
}

export function formatSeconds(totalSecs: number): string {
  if (totalSecs <= 0) return '0s';
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = Math.floor(totalSecs % 60);
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export const clock = new TickClock();

/** Simple polling helper: calls `fn` once immediately, then every `ms` ms. */
export function pollEvery(ms: number, fn: () => void): () => void {
  fn();
  const id = setInterval(fn, ms);
  return () => clearInterval(id);
}
