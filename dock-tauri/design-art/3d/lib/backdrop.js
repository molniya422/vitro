/* 共用舞台背景与地面。展厅和风格页保持同一套光照环境。 */

import * as THREE from "../../vendor/three/three.module.js";

export function backdropTexture() {
  const S = 1024;
  const c = document.createElement("canvas");
  c.width = S;
  c.height = S;
  const g = c.getContext("2d");
  const grd = g.createRadialGradient(340, 220, 20, 512, 540, 840);
  grd.addColorStop(0, "#26334c");
  grd.addColorStop(0.42, "#131a27");
  grd.addColorStop(1, "#06090f");
  g.fillStyle = grd;
  g.fillRect(0, 0, S, S);
  const tex = new THREE.CanvasTexture(c);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

function floorFade() {
  const S = 512;
  const c = document.createElement("canvas");
  c.width = S;
  c.height = S;
  const g = c.getContext("2d");
  const grd = g.createRadialGradient(256, 256, 30, 256, 256, 252);
  grd.addColorStop(0, "rgba(255,255,255,1)");
  grd.addColorStop(0.55, "rgba(255,255,255,0.55)");
  grd.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = grd;
  g.fillRect(0, 0, S, S);
  return new THREE.CanvasTexture(c);
}

export function createFloor(y = -1.35) {
  const floor = new THREE.Mesh(
    new THREE.PlaneGeometry(26, 26),
    new THREE.MeshStandardMaterial({
      color: 0x121a28,
      roughness: 0.5,
      metalness: 0.35,
      transparent: true,
      alphaMap: floorFade(),
      depthWrite: false,
      envMapIntensity: 0.7,
    })
  );
  floor.rotation.x = -Math.PI / 2;
  floor.position.y = y;
  return floor;
}
