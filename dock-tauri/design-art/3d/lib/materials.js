/* 共享材质：玻璃、图标面、图标边、面板、字形、强调。 */

import * as THREE from "../../vendor/three/three.module.js";

export function glass(opts = {}) {
  return new THREE.MeshPhysicalMaterial({
    color: opts.color ?? 0x2a3850,
    metalness: opts.metalness ?? 0.1,
    roughness: opts.roughness ?? 0.06,
    transparent: true,
    opacity: opts.opacity ?? 0.5,
    clearcoat: 1,
    clearcoatRoughness: 0.05,
    envMapIntensity: opts.envMapIntensity ?? 2.2,
    emissive: opts.emissive ?? 0x0c1626,
    emissiveIntensity: opts.emissiveIntensity ?? 0.55,
    depthWrite: false,
  });
}

/* 图标正面：挤出几何的 UV 等于形状坐标，所以要按边长归一化 */
export function tileFace(texture, size) {
  texture.repeat.set(1 / size, 1 / size);
  texture.offset.set(0.5, 0.5);
  return new THREE.MeshPhysicalMaterial({
    map: texture,
    roughness: 0.42,
    metalness: 0.02,
    clearcoat: 0.45,
    clearcoatRoughness: 0.28,
    envMapIntensity: 1.0,
  });
}

export function tileEdge(color) {
  return new THREE.MeshPhysicalMaterial({
    color: new THREE.Color(color).multiplyScalar(0.72),
    roughness: 0.5,
    metalness: 0.02,
    envMapIntensity: 0.8,
  });
}

/* 磨砂：高粗糙度加低透明度。软渲染下比 transmission 可靠 */
export function frostMaterial() {
  return new THREE.MeshPhysicalMaterial({
    color: 0xf2f6ff,
    transparent: true,
    opacity: 0.42,
    roughness: 0.62,
    metalness: 0,
    clearcoat: 0.5,
    clearcoatRoughness: 0.42,
    envMapIntensity: 1.2,
    depthWrite: false,
  });
}

/* 面板、菜单这类实色表面 */
export function panelSurface(opts = {}) {
  return new THREE.MeshPhysicalMaterial({
    color: opts.color ?? 0x141826,
    roughness: opts.roughness ?? 0.28,
    metalness: opts.metalness ?? 0.15,
    clearcoat: opts.clearcoat ?? 0.6,
    clearcoatRoughness: 0.2,
    envMapIntensity: opts.envMapIntensity ?? 0.9,
  });
}

export function glyphMat(color = 0xeef2fb) {
  return new THREE.MeshStandardMaterial({
    color,
    roughness: 0.32,
    metalness: 0.04,
    emissive: 0x141c2c,
    emissiveIntensity: 0.8,
  });
}

export function accentMat(color = 0x7c9be0, emissive = 0x2a3f66) {
  return new THREE.MeshStandardMaterial({
    color,
    roughness: 0.35,
    metalness: 0.05,
    emissive,
    emissiveIntensity: 0.7,
  });
}

export function flatMat(color, opacity = 1) {
  return new THREE.MeshStandardMaterial({
    color,
    roughness: 0.45,
    metalness: 0.05,
    transparent: opacity < 1,
    opacity,
  });
}
