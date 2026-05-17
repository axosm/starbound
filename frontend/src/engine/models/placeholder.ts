/**
 * Placeholder 3D models with Three.js AnimationMixer.
 *
 * Each model is a simple geometric shape with:
 *   - idle   animation (gentle bob / rotation)
 *   - walk   animation (forward lean + limb swing via morph targets or bone hack)
 *   - fight  animation (rapid shake)
 *
 * When real GLTF assets are ready, replace `createPlaceholder*` with
 * a GLTFLoader call and wire up the same animation names.
 */

import * as THREE from 'three';

export type AnimationState = 'idle' | 'walk' | 'fight';

export interface GameModel {
  group:   THREE.Group;
  mixer:   THREE.AnimationMixer;
  actions: Record<AnimationState, THREE.AnimationAction>;
  setState(s: AnimationState): void;
  update(delta: number): void;
  dispose(): void;
}

// ─── Keyframe helpers ────────────────────────────────────────

function makeClip(
  name:   string,
  tracks: THREE.KeyframeTrack[],
  duration = 2,
): THREE.AnimationClip {
  return new THREE.AnimationClip(name, duration, tracks);
}

/** Build a simple Y-bob idle clip for a given object. */
function idleClip(target: THREE.Object3D): THREE.AnimationClip {
  const yTrack = new THREE.VectorKeyframeTrack(
    `${target.name}.position[y]`,
    [0, 1, 2],
    [
      target.position.y,
      target.position.y + 0.08,
      target.position.y,
    ],
  );
  return makeClip('idle', [yTrack], 2);
}

/** Build a Z-rotation "fight shake" clip. */
function fightClip(target: THREE.Object3D): THREE.AnimationClip {
  const rz = new THREE.NumberKeyframeTrack(
    `${target.name}.rotation[z]`,
    [0, 0.1, 0.2, 0.3, 0.4],
    [0, 0.2, 0, -0.2, 0],
  );
  return makeClip('fight', [rz], 0.4);
}

/** Build a walk clip: forward lean on X then back. */
function walkClip(target: THREE.Object3D): THREE.AnimationClip {
  const rx = new THREE.NumberKeyframeTrack(
    `${target.name}.rotation[x]`,
    [0, 0.5, 1],
    [0, -0.2, 0],
  );
  return makeClip('walk', [rx], 1);
}

// ─── Generic placeholder builder ────────────────────────────

function buildModel(
  mesh: THREE.Mesh,
  color: number,
): GameModel {
  mesh.name = 'root';
  mesh.castShadow    = true;
  mesh.receiveShadow = true;

  const mat = new THREE.MeshStandardMaterial({ color, roughness: 0.7, metalness: 0.1 });
  mesh.material = mat;

  const group = new THREE.Group();
  group.add(mesh);

  const mixer = new THREE.AnimationMixer(mesh);

  const clips = {
    idle:  idleClip(mesh),
    walk:  walkClip(mesh),
    fight: fightClip(mesh),
  };

  const actions: Record<AnimationState, THREE.AnimationAction> = {
    idle:  mixer.clipAction(clips.idle),
    walk:  mixer.clipAction(clips.walk),
    fight: mixer.clipAction(clips.fight),
  };

  actions.idle.play();
  let current: AnimationState = 'idle';

  const model: GameModel = {
    group,
    mixer,
    actions,

    setState(next: AnimationState) {
      if (next === current) return;
      actions[current].fadeOut(0.2);
      actions[next].reset().fadeIn(0.2).play();
      current = next;
    },

    update(delta: number) {
      mixer.update(delta);
    },

    dispose() {
      mixer.stopAllAction();
      mesh.geometry.dispose();
      (mesh.material as THREE.Material).dispose();
    },
  };

  return model;
}

// ─── Public factory functions ────────────────────────────────

/** Unit model: capsule-ish cylinder with a sphere head. */
export function createUnitModel(unitType: string): GameModel {
  const colorMap: Record<string, number> = {
    soldier:    0x4a9eff,
    archer:     0x22c55e,
    cavalry:    0xf59e0b,
    catapult:   0xef4444,
    fighter:    0xa78bfa,
    bomber:     0xf97316,
    battleship: 0x64748b,
    transport:  0x94a3b8,
  };
  const color = colorMap[unitType] ?? 0x888888;

  // Body
  const bodyGeo  = new THREE.CylinderGeometry(0.2, 0.25, 0.6, 8);
  const body     = new THREE.Mesh(bodyGeo);
  body.position.y = 0.3;
  body.name = 'root';

  // Head
  const headGeo  = new THREE.SphereGeometry(0.18, 8, 8);
  const headMesh = new THREE.Mesh(headGeo, new THREE.MeshStandardMaterial({ color }));
  headMesh.position.y = 0.72;

  const group = new THREE.Group();
  group.add(body, headMesh);

  const mat = new THREE.MeshStandardMaterial({ color, roughness: 0.7, metalness: 0.1 });
  body.material = mat;

  const mixer = new THREE.AnimationMixer(body);
  body.castShadow = true;

  const clips = {
    idle:  idleClip(body),
    walk:  walkClip(body),
    fight: fightClip(body),
  };
  const actions: Record<AnimationState, THREE.AnimationAction> = {
    idle:  mixer.clipAction(clips.idle),
    walk:  mixer.clipAction(clips.walk),
    fight: mixer.clipAction(clips.fight),
  };
  actions.idle.play();
  let current: AnimationState = 'idle';

  return {
    group,
    mixer,
    actions,
    setState(next) {
      if (next === current) return;
      actions[current].fadeOut(0.2);
      actions[next].reset().fadeIn(0.2).play();
      current = next;
    },
    update(delta) { mixer.update(delta); },
    dispose() {
      mixer.stopAllAction();
      bodyGeo.dispose();
      headGeo.dispose();
      mat.dispose();
    },
  };
}

/** Building model: box with optional tower. */
export function createBuildingModel(buildingType: string): GameModel {
  const colorMap: Record<string, number> = {
    town_center:    0xd97706,
    lumber_mill:    0x16a34a,
    quarry:         0x78716c,
    farm:           0x84cc16,
    mine:           0x57534e,
    barracks:       0xdc2626,
    storage:        0xca8a04,
    wall:           0x6b7280,
    watchtower:     0x0ea5e9,
    lab:            0x8b5cf6,
    launch_pad:     0x06b6d4,
    space_dock:     0x0284c7,
    stargate:       0xd946ef,
    flying_fortress:0xe11d48,
  };
  const color = colorMap[buildingType] ?? 0xaaaaaa;
  const geo   = new THREE.BoxGeometry(0.7, 0.5, 0.7);
  const mesh  = new THREE.Mesh(geo);
  mesh.position.y = 0.25;
  return buildModel(mesh, color);
}

/** Space ship model: elongated cylinder. */
export function createShipModel(unitType: string): GameModel {
  const color = unitType === 'battleship' ? 0x334155 : 0x0ea5e9;
  const geo   = new THREE.CylinderGeometry(0.1, 0.2, 0.8, 6);
  const mesh  = new THREE.Mesh(geo);
  mesh.rotation.x = Math.PI / 2;
  return buildModel(mesh, color);
}
