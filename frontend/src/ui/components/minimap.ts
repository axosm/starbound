/**
 * Minimap — a small 2D canvas overlay showing the current planet's
 * tile ownership distribution, or the solar system from above.
 */

export class Minimap {
  private canvas: HTMLCanvasElement;
  private ctx:    CanvasRenderingContext2D;

  constructor(container: HTMLElement) {
    this.canvas = document.createElement('canvas');
    this.canvas.id     = 'minimap';
    this.canvas.width  = 160;
    this.canvas.height = 160;
    container.appendChild(this.canvas);
    this.ctx = this.canvas.getContext('2d')!;
    injectCSS();
  }

  /** Draw a dot-grid overview of tile ownership. */
  drawPlanetView(
    tiles: { u: number; v: number; tile_type: string; owner_player_id: number | null }[],
    myPlayerId: number,
  ) {
    const { ctx } = this;
    ctx.clearRect(0, 0, 160, 160);
    ctx.fillStyle = '#020408';
    ctx.fillRect(0, 0, 160, 160);

    const COLORS: Record<string, string> = {
      plains:   '#86efac', forest: '#166534', mountain: '#78716c',
      desert:   '#fde68a', snow:   '#f1f5f9', lava:     '#ef4444',
      water:    '#38bdf8', ocean:  '#1e40af',
    };

    const maxU = Math.max(...tiles.map(t => t.u), 1);
    const maxV = Math.max(...tiles.map(t => t.v), 1);

    for (const t of tiles) {
      const x = (t.u / maxU) * 150 + 5;
      const y = (t.v / maxV) * 150 + 5;
      ctx.fillStyle = COLORS[t.tile_type] ?? '#888';
      ctx.fillRect(x, y, 6, 6);
      // Ownership border
      if (t.owner_player_id === myPlayerId) {
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 1;
        ctx.strokeRect(x, y, 6, 6);
      } else if (t.owner_player_id !== null) {
        ctx.strokeStyle = '#ef4444';
        ctx.lineWidth = 1;
        ctx.strokeRect(x, y, 6, 6);
      }
    }
  }

  clear() {
    this.ctx.clearRect(0, 0, 160, 160);
    this.ctx.fillStyle = '#020408';
    this.ctx.fillRect(0, 0, 160, 160);
  }

  destroy() { this.canvas.remove(); }
}

function injectCSS() {
  if (document.getElementById('minimap-css')) return;
  const s = document.createElement('style');
  s.id = 'minimap-css';
  s.textContent = `
    #minimap {
      position: fixed; bottom: 3.5rem; right: 1rem;
      width: 160px; height: 160px;
      border: 1px solid #1e3a5f; border-radius: 6px;
      z-index: 150;
      image-rendering: pixelated;
    }
  `;
  document.head.appendChild(s);
}
