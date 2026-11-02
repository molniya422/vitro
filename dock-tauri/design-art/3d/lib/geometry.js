/* 共享几何：超椭圆、药丸、圆角面板。所有三维组件都从这里取形。 */

import * as THREE from "../../vendor/three/three.module.js";
import { RoundedBoxGeometry } from "../../vendor/three/addons/geometries/RoundedBoxGeometry.js";

export const SQUIRCLE_N = 5;

export function squircleShape(size, steps = 128) {
  const r = size / 2;
  const shape = new THREE.Shape();
  for (let i = 0; i <= steps; i++) {
    const t = (i / steps) * Math.PI * 2;
    const c = Math.cos(t);
    const s = Math.sin(t);
    const x = Math.sign(c) * Math.pow(Math.abs(c), 2 / SQUIRCLE_N) * r;
    const y = Math.sign(s) * Math.pow(Math.abs(s), 2 / SQUIRCLE_N) * r;
    if (i === 0) shape.moveTo(x, y);
    else shape.lineTo(x, y);
  }
  shape.closePath();
  return shape;
}

export function stadiumShape(w, h) {
  const r = h / 2;
  const shape = new THREE.Shape();
  shape.moveTo(-w / 2 + r, -r);
  shape.lineTo(w / 2 - r, -r);
  shape.absarc(w / 2 - r, 0, r, -Math.PI / 2, Math.PI / 2, false);
  shape.lineTo(-w / 2 + r, r);
  shape.absarc(-w / 2 + r, 0, r, Math.PI / 2, Math.PI * 1.5, false);
  shape.closePath();
  return shape;
}

function extrude(shape, depth, bevel, curveSegments) {
  const geo = new THREE.ExtrudeGeometry(shape, {
    depth,
    bevelEnabled: bevel > 0,
    bevelThickness: bevel,
    bevelSize: bevel,
    bevelSegments: 3,
    curveSegments,
  });
  geo.center();
  geo.computeBoundingBox();
  return geo;
}

export function squircleGeo(size, depth = 0.1, bevel = 0.012) {
  return extrude(squircleShape(size), depth, bevel, 1);
}

export function stadiumGeo(w, h, depth = 0.2, bevel = 0.03) {
  return extrude(stadiumShape(w, h), depth, bevel, 32);
}

export function panelGeo(w, h, depth = 0.14, radius = 0.12) {
  return new RoundedBoxGeometry(w, h, depth, 4, radius);
}

export function barGeo(w, h, depth) {
  return new THREE.BoxGeometry(w, h, depth);
}

export function ringGeo(radius, tube) {
  return new THREE.TorusGeometry(radius, tube, 18, 64);
}

/* 超椭圆环：裸排版的光环、角标的外描边都用它 */
export function squircleRingGeo(size, thickness = 0.01, depth = 0.02) {
  const shape = squircleShape(size);
  shape.holes.push(squircleShape(size - thickness * 2));
  const geo = new THREE.ExtrudeGeometry(shape, {
    depth,
    bevelEnabled: false,
    curveSegments: 1,
  });
  geo.center();
  return geo;
}
