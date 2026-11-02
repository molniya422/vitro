/* Dock 三维视觉稿
   同一套参数的三维版本：22.37% 超椭圆、玻璃四层、1.6 放大曲线。
   打包成 IIFE 后无需本地服务器，双击 index.html 即可打开。 */

import * as THREE from "../vendor/three/three.module.js";
import { OrbitControls } from "../vendor/three/addons/controls/OrbitControls.js";
import { RoomEnvironment } from "../vendor/three/addons/environments/RoomEnvironment.js";

const canvas = document.getElementById("stage");
const hint = document.getElementById("hint");
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const TILE_COUNT = 5;
const TILE_SIZE = 1;
const TILE_GAP = 0.09;
const PITCH = TILE_SIZE + TILE_GAP;
const DOCK_W = TILE_COUNT * PITCH + 0.7;
const DOCK_D = 1.26;
const DOCK_T = 0.42;

const PALETTE = [0x4f6bb0, 0xb8543d, 0x2c7360, 0x8f4a4a, 0x525c74];
const GLYPHS = ["ring", "bars", "triangle", "dot", "cross"];

/* ---------------- renderer ---------------- */

let renderer;
try {
  renderer = new THREE.WebGLRenderer({
    canvas,
    antialias: true,
    powerPreference: "high-performance",
  });
} catch (error) {
  document.body.classList.add("no-webgl");
  throw error;
}

renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.12;
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;

/* ---------------- backdrop ---------------- */

function backdropTexture() {
  const size = 1024;
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const g = c.getContext("2d");
  const grd = g.createRadialGradient(330, 210, 20, 512, 520, 830);
  grd.addColorStop(0, "#26334c");
  grd.addColorStop(0.4, "#141a27");
  grd.addColorStop(1, "#070a11");
  g.fillStyle = grd;
  g.fillRect(0, 0, size, size);
  const tex = new THREE.CanvasTexture(c);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

function alphaFade() {
  const size = 512;
  const c = document.createElement("canvas");
  c.width = size;
  c.height = size;
  const g = c.getContext("2d");
  const grd = g.createRadialGradient(256, 256, 30, 256, 256, 252);
  grd.addColorStop(0, "rgba(255,255,255,1)");
  grd.addColorStop(0.5, "rgba(255,255,255,0.6)");
  grd.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = grd;
  g.fillRect(0, 0, size, size);
  return new THREE.CanvasTexture(c);
}

const scene = new THREE.Scene();
scene.background = backdropTexture();

/* ---------------- camera ---------------- */

const camera = new THREE.PerspectiveCamera(32, 1, 0.1, 80);
const controls = new OrbitControls(camera, canvas);
controls.target.set(0, 0.52, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.07;
controls.enablePan = false;
controls.minPolarAngle = THREE.MathUtils.degToRad(50);
controls.maxPolarAngle = THREE.MathUtils.degToRad(86);
controls.minDistance = 4.2;
controls.maxDistance = 18;
controls.autoRotate = !reduceMotion;
controls.autoRotateSpeed = 0.45;
controls.rotateSpeed = 0.7;

/* ---------------- environment and lights ---------------- */

const pmrem = new THREE.PMREMGenerator(renderer);
scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.05).texture;
pmrem.dispose();

const key = new THREE.DirectionalLight(0xe6eeff, 3.1);
key.position.set(-4.5, 7, 5.5);
key.castShadow = true;
key.shadow.mapSize.set(1024, 1024);
key.shadow.camera.near = 1;
key.shadow.camera.far = 22;
key.shadow.camera.left = -6;
key.shadow.camera.right = 6;
key.shadow.camera.top = 6;
key.shadow.camera.bottom = -6;
key.shadow.radius = 4;
key.shadow.bias = -0.0006;
scene.add(key);

const rim = new THREE.DirectionalLight(0x7c9be0, 2.6);
rim.position.set(5.5, 3.4, -4.5);
scene.add(rim);

const bounce = new THREE.HemisphereLight(0xaec4ff, 0x080b12, 0.55);
scene.add(bounce);

/* ---------------- floor ---------------- */

const floor = new THREE.Mesh(
  new THREE.PlaneGeometry(26, 26),
  new THREE.MeshStandardMaterial({
    color: 0x121a28,
    roughness: 0.55,
    metalness: 0.3,
    transparent: true,
    alphaMap: alphaFade(),
    depthWrite: false,
    envMapIntensity: 0.7,
  })
);
floor.rotation.x = -Math.PI / 2;
floor.position.y = -1.5;
floor.receiveShadow = true;
scene.add(floor);

/* ---------------- geometry ---------------- */

function pillShape(w, h) {
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

function squircleShape(size) {
  const shape = new THREE.Shape();
  const r = size / 2;
  const n = 5;
  const steps = 128;
  for (let i = 0; i <= steps; i++) {
    const t = (i / steps) * Math.PI * 2;
    const c = Math.cos(t);
    const s = Math.sin(t);
    const x = Math.sign(c) * Math.pow(Math.abs(c), 2 / n) * r;
    const y = Math.sign(s) * Math.pow(Math.abs(s), 2 / n) * r;
    if (i === 0) shape.moveTo(x, y);
    else shape.lineTo(x, y);
  }
  shape.closePath();
  return shape;
}

const pillGeo = new THREE.ExtrudeGeometry(pillShape(DOCK_W, DOCK_D), {
  depth: DOCK_T,
  bevelEnabled: true,
  bevelThickness: 0.06,
  bevelSize: 0.06,
  bevelSegments: 5,
  curveSegments: 32,
});
pillGeo.center();
pillGeo.rotateX(-Math.PI / 2);
pillGeo.computeBoundingBox();

const tileGeo = new THREE.ExtrudeGeometry(squircleShape(TILE_SIZE), {
  depth: 0.14,
  bevelEnabled: true,
  bevelThickness: 0.05,
  bevelSize: 0.05,
  bevelSegments: 4,
  curveSegments: 1,
});
tileGeo.center();

/* ---------------- dock ---------------- */

const dock = new THREE.Group();
scene.add(dock);

// 不用 transmission：软件渲染下透射通道会返回全黑。
// 半透明 + 清漆在真实 GPU 与软渲染上表现一致，也便宜得多。
const glass = new THREE.MeshPhysicalMaterial({
  color: 0x2a3850,
  metalness: 0.1,
  roughness: 0.06,
  transparent: true,
  opacity: 0.52,
  clearcoat: 1,
  clearcoatRoughness: 0.04,
  envMapIntensity: 2.2,
  emissive: 0x0c1626,
  emissiveIntensity: 0.55,
  depthWrite: false,
});

const pill = new THREE.Mesh(pillGeo, glass);
pill.castShadow = true;
dock.add(pill);

const pillTop = pillGeo.boundingBox.max.y;
const REST_Y = pillTop + TILE_SIZE / 2 + 0.05;

const glyphMat = new THREE.MeshStandardMaterial({
  color: 0xeef2fb,
  roughness: 0.32,
  metalness: 0.04,
  emissive: 0x141c2c,
  emissiveIntensity: 0.8,
});

function makeGlyph(kind) {
  const group = new THREE.Group();

  if (kind === "ring") {
    group.add(new THREE.Mesh(new THREE.TorusGeometry(0.17, 0.032, 20, 64), glyphMat));
  } else if (kind === "bars") {
    [-0.12, 0, 0.12].forEach((y, i) => {
      const bar = new THREE.Mesh(new THREE.BoxGeometry(0.36 - i * 0.07, 0.042, 0.042), glyphMat);
      bar.position.y = y;
      group.add(bar);
    });
  } else if (kind === "triangle") {
    group.add(new THREE.Mesh(new THREE.ConeGeometry(0.19, 0.31, 3), glyphMat));
  } else if (kind === "dot") {
    group.add(new THREE.Mesh(new THREE.SphereGeometry(0.13, 24, 16), glyphMat));
  } else {
    group.add(new THREE.Mesh(new THREE.BoxGeometry(0.34, 0.058, 0.05), glyphMat));
    group.add(new THREE.Mesh(new THREE.BoxGeometry(0.058, 0.34, 0.05), glyphMat));
  }

  group.position.z = 0.13;
  return group;
}

const tiles = [];

for (let i = 0; i < TILE_COUNT; i++) {
  const group = new THREE.Group();
  const material = new THREE.MeshPhysicalMaterial({
    color: PALETTE[i],
    roughness: 0.5,
    metalness: 0.02,
    clearcoat: 0.35,
    clearcoatRoughness: 0.3,
    envMapIntensity: 0.95,
  });

  const body = new THREE.Mesh(tileGeo, material);
  body.castShadow = true;
  body.receiveShadow = true;

  const x = (i - (TILE_COUNT - 1) / 2) * PITCH;
  group.position.set(x, REST_Y, 0.05);
  group.userData.baseX = x;
  group.userData.restY = REST_Y;
  group.add(body, makeGlyph(GLYPHS[i]));

  dock.add(group);
  tiles.push(group);
}

/* ---------------- hover magnification ---------------- */

const raycaster = new THREE.Raycaster();
const pointer = new THREE.Vector2();
const hoverPlane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
let pointerX = null;

function onPointerMove(event) {
  const rect = canvas.getBoundingClientRect();
  pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
  pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
  raycaster.setFromCamera(pointer, camera);
  const hit = new THREE.Vector3();
  pointerX = raycaster.ray.intersectPlane(hoverPlane, hit) ? hit.x : null;
}

canvas.addEventListener("pointermove", onPointerMove, { passive: true });
canvas.addEventListener("pointerleave", () => {
  pointerX = null;
});

/* ---------------- framing ---------------- */

let lastAspect = 0;

function fit() {
  const half = THREE.MathUtils.degToRad(camera.fov * 0.5);
  const needWidth = (DOCK_W + 1.9) / (2 * Math.tan(half) * camera.aspect);
  const needHeight = 3.5 / (2 * Math.tan(half));
  const distance = Math.max(needWidth, needHeight, 5.4);
  const dir = new THREE.Vector3(0.42, 0.52, 1).normalize();
  camera.position.copy(controls.target).add(dir.multiplyScalar(distance));
  controls.update();
  lastAspect = camera.aspect;
}

function resize() {
  const w = canvas.clientWidth || window.innerWidth;
  const h = canvas.clientHeight || window.innerHeight;
  renderer.setSize(w, h, false);
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  if (Math.abs(camera.aspect - lastAspect) > 0.28 || lastAspect === 0) fit();
}

window.addEventListener("resize", resize);

/* ---------------- ui ---------------- */

const rotateBtn = document.getElementById("btn-rotate");
const resetBtn = document.getElementById("btn-reset");

rotateBtn.setAttribute("aria-pressed", String(controls.autoRotate));
rotateBtn.addEventListener("click", () => {
  controls.autoRotate = !controls.autoRotate;
  rotateBtn.setAttribute("aria-pressed", String(controls.autoRotate));
});

resetBtn.addEventListener("click", () => {
  fit();
});

controls.addEventListener("start", () => hint.classList.add("is-hidden"));

/* ---------------- loop ---------------- */

const clock = new THREE.Clock();
let raf = 0;

function frame() {
  const dt = Math.min(clock.getDelta(), 0.05);
  const t = clock.elapsedTime;

  if (!reduceMotion) {
    dock.position.y = Math.sin(t * 0.75) * 0.035;
    dock.rotation.z = Math.sin(t * 0.5) * 0.006;
  }

  for (let i = 0; i < tiles.length; i++) {
    const tile = tiles[i];
    let lift = 0;
    let scale = 1;

    if (pointerX !== null) {
      const d = Math.abs(tile.userData.baseX - pointerX) / PITCH;
      const weight = Math.exp(-Math.pow(d / 0.85, 4));
      lift = 0.2 * weight;
      scale = 1 + 0.24 * weight;
    }

    const bob = reduceMotion ? 0 : Math.sin(t * 0.9 + i * 0.7) * 0.008;
    const wantY = tile.userData.restY + lift + bob;
    tile.position.y += (wantY - tile.position.y) * Math.min(1, dt * 9);

    const current = tile.scale.x;
    const next = current + (scale - current) * Math.min(1, dt * 10);
    tile.scale.setScalar(next);
  }

  controls.update();
  renderer.render(scene, camera);
  raf = requestAnimationFrame(frame);
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    cancelAnimationFrame(raf);
    raf = 0;
  } else if (!raf) {
    clock.getDelta();
    raf = requestAnimationFrame(frame);
  }
});

resize();
fit();
raf = requestAnimationFrame(frame);
