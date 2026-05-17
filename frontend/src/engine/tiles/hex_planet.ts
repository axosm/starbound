/**
 * Flat-topped hexagonal grid mapped onto a sphere (Civ 6 approach).
 *
 * Layout:
 *   - For a given subdivision N, there are ~5N² hexagonal faces.
 *   - We use offset coordinates (face, u, v) matching the DB schema.
 *   - Each tile is a flat hexagonal prism positioned on a sphere surface.
 *
 * This is a simplified projection: a grid wrapped on a UV sphere,
 * with tiles placed at (θ, φ) grid positions. Poles have distortion
 * (same as Civ 6) which is acceptable for gameplay.
 */

import * as THREE from 'three';
import type { TileDto } from '../../types/api';
import {
  createBuildingModel,
  createUnitModel,
  type GameModel,
} from '../models/placeholder';

const TILE_COLORS: Record<string, number> = {
  plains:   0x86efac,
  forest:   0x166534,
  mountain: 0x78716c,
  desert:   0xfde68a,
  snow:     0xf1f5f9,
  lava:     0xef4444,
  water:    0x38bdf8,
  ocean:    0x1e40af,
};

export interface TileMesh {
  tileId:  number;
  mesh:    THREE.Mesh;
  model?:  GameModel;
}

export class HexPlanet {
  private scene:     THREE.Scene;
  private radius:    number;
  private tileMeshes: Map<number, TileMesh> = new Map();
  private modelClock = new THREE.Clock();

  constructor(scene: THREE.Scene, radius = 5) {
    this.scene  = scene;
    this.radius = radius;
  }

  /** Build or rebuild the tile grid from a PlanetViewResponse tiles array. */
  buildTiles(tiles: TileDto[], subdivision: number) {
    this.clear();

    const cols = subdivision * 4;   // columns around equator
    const rows = subdivision * 2;   // rows pole-to-pole

    for (const tile of tiles) {
      // Map (face, u, v) to spherical coordinates
      const theta = (tile.u / cols) * Math.PI * 2;
      const phi   = (tile.v / rows) * Math.PI - Math.PI / 2;

      const tileSize = (Math.PI * 2) / cols * this.radius * 0.9;
      const geo = this.hexGeo(tileSize * 0.5);

      const color = TILE_COLORS[tile.tile_type] ?? 0x888888;
      const mat   = new THREE.MeshStandardMaterial({
        color,
        roughness: 0.9,
        metalness: 0.0,
      });

      if (tile.owner_player_id !== null) {
        mat.emissive    = new THREE.Color(0x002244);
        mat.emissiveIntensity = 0.3;
      }

      const mesh = new THREE.Mesh(geo, mat);
      mesh.castShadow    = true;
      mesh.receiveShadow = true;

      // Position on sphere surface
      const x = this.radius * Math.cos(phi) * Math.cos(theta);
      const y = this.radius * Math.sin(phi);
      const z = this.radius * Math.cos(phi) * Math.sin(theta);
      mesh.position.set(x, y, z);

      // Orient normal to sphere surface
      mesh.lookAt(0, 0, 0);
      mesh.rotateX(Math.PI / 2);

      mesh.userData = { tileId: tile.id, tile };

      this.scene.add(mesh);
      const tm: TileMesh = { tileId: tile.id, mesh };

      // Place building model if present and constructed
      if (tile.building && !tile.building.construction_done_tick) {
        const bModel = createBuildingModel(tile.building.building_type);
        bModel.group.position.copy(mesh.position.clone().multiplyScalar(1.05));
        bModel.group.lookAt(0, 0, 0);
        this.scene.add(bModel.group);
        tm.model = bModel;
      }

      // Place unit models
      if (tile.units.length > 0) {
        const uModel = createUnitModel(tile.units[0].unit_type);
        const offset  = mesh.position.clone().multiplyScalar(1.08);
        uModel.group.position.copy(offset);
        uModel.group.lookAt(0, 0, 0);
        if (tile.units[0].player_id !== tile.owner_player_id) {
          uModel.setState('fight');
        } else {
          uModel.setState('idle');
        }
        this.scene.add(uModel.group);
        tm.model = uModel;
      }

      this.tileMeshes.set(tile.id, tm);
    }
  }

  /** Flat-top hexagonal prism geometry. */
  private hexGeo(radius: number): THREE.BufferGeometry {
    const shape = new THREE.Shape();
    for (let i = 0; i < 6; i++) {
      const angle = (Math.PI / 3) * i;
      const x = radius * Math.cos(angle);
      const y = radius * Math.sin(angle);
      if (i === 0) shape.moveTo(x, y);
      else shape.lineTo(x, y);
    }
    shape.closePath();

    return new THREE.ExtrudeGeometry(shape, {
      depth:        0.08,
      bevelEnabled: false,
    });
  }

  /** Update all model animations. Called every frame. */
  update() {
    const delta = this.modelClock.getDelta();
    for (const tm of this.tileMeshes.values()) {
      tm.model?.update(delta);
    }
  }

  /** Highlight a tile (e.g. selection). */
  highlight(tileId: number, color: number) {
    const tm = this.tileMeshes.get(tileId);
    if (!tm) return;
    (tm.mesh.material as THREE.MeshStandardMaterial).emissive = new THREE.Color(color);
    (tm.mesh.material as THREE.MeshStandardMaterial).emissiveIntensity = 0.6;
  }

  clearHighlights() {
    for (const tm of this.tileMeshes.values()) {
      const mat = tm.mesh.material as THREE.MeshStandardMaterial;
      mat.emissive.setHex(0x000000);
      mat.emissiveIntensity = 0;
    }
  }

  /** Raycast to find clicked tile. */
  pick(raycaster: THREE.Raycaster): TileDto | null {
    const meshes = [...this.tileMeshes.values()].map(t => t.mesh);
    const hits   = raycaster.intersectObjects(meshes);
    if (!hits.length) return null;
    return hits[0].object.userData.tile as TileDto;
  }

  clear() {
    for (const tm of this.tileMeshes.values()) {
      this.scene.remove(tm.mesh);
      tm.mesh.geometry.dispose();
      (tm.mesh.material as THREE.Material).dispose();
      if (tm.model) {
        this.scene.remove(tm.model.group);
        tm.model.dispose();
      }
    }
    this.tileMeshes.clear();
  }
}
