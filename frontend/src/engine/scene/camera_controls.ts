/**
 * Simple orbit camera controller.
 * (Using a manual implementation to avoid importing OrbitControls
 * which requires a separate import path in some Three.js versions.)
 */

import * as THREE from 'three';

export class OrbitCamera {
  camera:   THREE.PerspectiveCamera;
  target    = new THREE.Vector3();
  private spherical  = new THREE.Spherical(15, Math.PI / 4, 0);
  private isDragging = false;
  private lastMouse  = { x: 0, y: 0 };
  private canvas:    HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement, fov = 60) {
    this.canvas = canvas;
    this.camera = new THREE.PerspectiveCamera(
      fov,
      canvas.clientWidth / canvas.clientHeight,
      0.1,
      10_000,
    );
    this.updateCamera();
    this.bindEvents();
  }

  private updateCamera() {
    this.camera.position.setFromSpherical(this.spherical).add(this.target);
    this.camera.lookAt(this.target);
  }

  private bindEvents() {
    this.canvas.addEventListener('mousedown', e => {
      this.isDragging = true;
      this.lastMouse  = { x: e.clientX, y: e.clientY };
    });
    window.addEventListener('mouseup', () => { this.isDragging = false; });
    window.addEventListener('mousemove', e => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.lastMouse.x;
      const dy = e.clientY - this.lastMouse.y;
      this.lastMouse = { x: e.clientX, y: e.clientY };
      this.spherical.theta -= dx * 0.005;
      this.spherical.phi   -= dy * 0.005;
      this.spherical.phi    = Math.max(0.1, Math.min(Math.PI - 0.1, this.spherical.phi));
      this.updateCamera();
    });
    this.canvas.addEventListener('wheel', e => {
      this.spherical.radius *= 1 + e.deltaY * 0.001;
      this.spherical.radius  = Math.max(2, Math.min(500, this.spherical.radius));
      this.updateCamera();
      e.preventDefault();
    }, { passive: false });
  }

  setRadius(r: number) {
    this.spherical.radius = r;
    this.updateCamera();
  }

  update() { this.updateCamera(); }
}
