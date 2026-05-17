import * as THREE from 'three';

export interface RendererConfig {
  canvas:      HTMLCanvasElement;
  antialias?:  boolean;
  pixelRatio?: number;
}

export function createRenderer(cfg: RendererConfig): THREE.WebGLRenderer {
  const renderer = new THREE.WebGLRenderer({
    canvas:    cfg.canvas,
    antialias: cfg.antialias ?? true,
    alpha:     false,
  });
  renderer.setPixelRatio(cfg.pixelRatio ?? Math.min(window.devicePixelRatio, 2));
  renderer.setSize(cfg.canvas.clientWidth, cfg.canvas.clientHeight, false);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type    = THREE.PCFSoftShadowMap;
  renderer.toneMapping       = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;
  return renderer;
}

export function handleResize(
  renderer: THREE.WebGLRenderer,
  camera:   THREE.PerspectiveCamera,
) {
  const canvas = renderer.domElement;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  if (canvas.width !== w || canvas.height !== h) {
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }
}
