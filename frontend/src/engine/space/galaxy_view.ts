/**
 * Galaxy map — renders thousands of star systems as instanced points.
 * Uses THREE.Points for maximum performance.
 */

import * as THREE from 'three';

export interface StarSystem {
  id:   number;
  x:    number;
  y:    number;
  z:    number;
  seed: number;
}

export class GalaxyView {
  private scene:  THREE.Scene;
  private points: THREE.Points | null = null;

  constructor(scene: THREE.Scene) { this.scene = scene; }

  build(systems: StarSystem[]) {
    this.clear();

    const positions = new Float32Array(systems.length * 3);
    const colors    = new Float32Array(systems.length * 3);

    for (let i = 0; i < systems.length; i++) {
      const s = systems[i];
      positions[i * 3]     = s.x / 100;
      positions[i * 3 + 1] = s.y / 100;
      positions[i * 3 + 2] = s.z / 100;

      // Color by seed hash
      const r = ((s.seed >> 16) & 0xff) / 255;
      const g = ((s.seed >>  8) & 0xff) / 255;
      const b = ( s.seed        & 0xff) / 255;
      colors[i * 3]     = 0.6 + r * 0.4;
      colors[i * 3 + 1] = 0.6 + g * 0.4;
      colors[i * 3 + 2] = 0.7 + b * 0.3;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geo.setAttribute('color',    new THREE.BufferAttribute(colors, 3));

    const mat = new THREE.PointsMaterial({
      size:         0.3,
      vertexColors: true,
      sizeAttenuation: true,
    });

    this.points = new THREE.Points(geo, mat);
    this.scene.add(this.points);
  }

  clear() {
    if (this.points) {
      this.scene.remove(this.points);
      this.points.geometry.dispose();
      (this.points.material as THREE.Material).dispose();
      this.points = null;
    }
  }
}

/** Generate a background star field (decorative, no gameplay). */
export function addStarField(scene: THREE.Scene, count = 5000): THREE.Points {
  const positions = new Float32Array(count * 3);
  for (let i = 0; i < count; i++) {
    positions[i * 3]     = (Math.random() - 0.5) * 2000;
    positions[i * 3 + 1] = (Math.random() - 0.5) * 2000;
    positions[i * 3 + 2] = (Math.random() - 0.5) * 2000;
  }
  const geo  = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  const mat  = new THREE.PointsMaterial({ size: 0.5, color: 0xffffff, sizeAttenuation: true });
  const pts  = new THREE.Points(geo, mat);
  scene.add(pts);
  return pts;
}
