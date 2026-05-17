/**
 * Solar system view (Stellaris-style).
 *
 * - Central star rendered as a glowing sphere with a PointLight.
 * - Planets placed on circular orbits, rendered as textured spheres.
 * - Ships/units in space shown as small models at their (x,y,z) position.
 * - No physics: orbits are visual animation only.
 */

import * as THREE from 'three';
import type { PlanetSummary, UnitDto } from '../../types/api';
import { createShipModel, type GameModel } from '../models/placeholder';

export class SolarSystemView {
  private scene:      THREE.Scene;
  private planets:    THREE.Mesh[]  = [];
  private orbitLines: THREE.Line[]  = [];
  private shipModels: Map<number, GameModel> = new Map();
  private clock = new THREE.Clock();

  constructor(scene: THREE.Scene) { this.scene = scene; }

  build(planets: PlanetSummary[]) {
    this.clear();

    // Star
    const starGeo  = new THREE.SphereGeometry(1.2, 32, 32);
    const starMat  = new THREE.MeshBasicMaterial({ color: 0xffd700 });
    const star     = new THREE.Mesh(starGeo, starMat);
    this.scene.add(star);

    const starLight = new THREE.PointLight(0xfff5cc, 3, 200);
    this.scene.add(starLight);

    for (const p of planets) {
      const orbitRadius = Math.sqrt(p.x * p.x + p.y * p.y) / 10 + 3;
      this.addOrbitRing(orbitRadius);

      const planetGeo  = new THREE.SphereGeometry(0.4, 24, 24);
      const planetMat  = new THREE.MeshStandardMaterial({
        color: this.planetColor(p.planet_type),
        roughness: 0.8,
      });
      const mesh = new THREE.Mesh(planetGeo, planetMat);
      mesh.castShadow    = true;
      mesh.receiveShadow = true;
      mesh.userData      = { planet: p, orbitRadius, orbitAngle: Math.random() * Math.PI * 2 };
      mesh.position.set(orbitRadius, 0, 0);
      this.scene.add(mesh);
      this.planets.push(mesh);
    }
  }

  private addOrbitRing(radius: number) {
    const points: THREE.Vector3[] = [];
    for (let i = 0; i <= 64; i++) {
      const a = (i / 64) * Math.PI * 2;
      points.push(new THREE.Vector3(Math.cos(a) * radius, 0, Math.sin(a) * radius));
    }
    const geo  = new THREE.BufferGeometry().setFromPoints(points);
    const mat  = new THREE.LineBasicMaterial({ color: 0x334155, opacity: 0.4, transparent: true });
    const line = new THREE.Line(geo, mat);
    this.scene.add(line);
    this.orbitLines.push(line);
  }

  /** Place space units at their coordinates. */
  updateUnits(units: UnitDto[]) {
    const inSpace = units.filter(u => u.location_mode === 'in_space');
    // Remove stale
    for (const [id, model] of this.shipModels) {
      if (!inSpace.find(u => u.id === id)) {
        this.scene.remove(model.group);
        model.dispose();
        this.shipModels.delete(id);
      }
    }
    // Add new
    for (const u of inSpace) {
      if (!this.shipModels.has(u.id)) {
        const model = createShipModel(u.unit_type);
        this.scene.add(model.group);
        this.shipModels.set(u.id, model);
      }
      const model = this.shipModels.get(u.id)!;
      const x = (u.space_x ?? 0) / 10;
      const y = (u.space_z ?? 0) / 10;
      const z = (u.space_y ?? 0) / 10;
      model.group.position.set(x, y, z);
    }
  }

  update() {
    const delta = this.clock.getDelta();
    const elapsed = this.clock.getElapsedTime();
    // Animate orbits
    for (const mesh of this.planets) {
      const { orbitRadius, orbitAngle } = mesh.userData;
      const angle = orbitAngle + elapsed * 0.2;
      mesh.position.set(
        Math.cos(angle) * orbitRadius,
        0,
        Math.sin(angle) * orbitRadius,
      );
    }
    // Update ship animations
    for (const model of this.shipModels.values()) {
      model.update(delta);
    }
  }

  /** Raycast to find clicked planet. */
  pick(raycaster: THREE.Raycaster): PlanetSummary | null {
    const hits = raycaster.intersectObjects(this.planets);
    if (!hits.length) return null;
    return hits[0].object.userData.planet as PlanetSummary;
  }

  private planetColor(type: string): number {
    const map: Record<string, number> = {
      terrestrial: 0x4ade80,
      ocean:       0x38bdf8,
      desert:      0xfbbf24,
      ice:         0xe0f2fe,
      lava:        0xf87171,
      gas_giant:   0xfb923c,
      barren:      0x78716c,
    };
    return map[type] ?? 0x888888;
  }

  clear() {
    for (const m of this.planets) {
      this.scene.remove(m);
      m.geometry.dispose();
      (m.material as THREE.Material).dispose();
    }
    this.planets = [];
    for (const l of this.orbitLines) {
      this.scene.remove(l);
      l.geometry.dispose();
    }
    this.orbitLines = [];
    for (const m of this.shipModels.values()) {
      this.scene.remove(m.group);
      m.dispose();
    }
    this.shipModels.clear();
  }
}
