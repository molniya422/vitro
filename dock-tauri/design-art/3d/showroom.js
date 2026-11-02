/* 三维资产展厅
   把整套美术资源按同一套构件立起来：Dock 主栏、插件磁贴、右键菜单、设置面板、品牌标记。
   全部来自 lib/，与可挂载的 Dock 层共用几何、材质与贴图。 */

import * as THREE from "../vendor/three/three.module.js";
import { OrbitControls } from "../vendor/three/addons/controls/OrbitControls.js";
import { RoomEnvironment } from "../vendor/three/addons/environments/RoomEnvironment.js";
import { panelGeo, ringGeo, squircleGeo, stadiumGeo } from "./lib/geometry.js";
import { accentMat, flatMat, glass, panelSurface } from "./lib/materials.js";
import { badgeTexture, labelTexture } from "./lib/textures.js";
import { createTile } from "./lib/tiles.js";
import { createFloor, backdropTexture } from "./lib/backdrop.js";
import { textSprite } from "./lib/labels.js";
import { buildDock } from "./lib/dock-piece.js";

const canvas = document.getElementById("scene");
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.08;

const scene = new THREE.Scene();
scene.background = backdropTexture();

const camera = new THREE.PerspectiveCamera(30, 1, 0.1, 80);
const controls = new OrbitControls(camera, canvas);
controls.target.set(0, 1.45, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.07;
controls.enablePan = false;
controls.minDistance = 7;
controls.maxDistance = 26;
controls.minPolarAngle = THREE.MathUtils.degToRad(48);
controls.maxPolarAngle = THREE.MathUtils.degToRad(84);
controls.autoRotate = !reduceMotion;
controls.autoRotateSpeed = 0.35;

const pmrem = new THREE.PMREMGenerator(renderer);
scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.05).texture;
pmrem.dispose();

const key = new THREE.DirectionalLight(0xeaf1ff, 3);
key.position.set(-4, 7, 6);
scene.add(key);

const rim = new THREE.DirectionalLight(0x7c9be0, 2.4);
rim.position.set(5, 3, -5);
scene.add(rim);

scene.add(new THREE.HemisphereLight(0xb6c9ff, 0x080b12, 0.55));

/* ---------------- 背景与地面 ---------------- */

scene.add(createFloor());

/* ---------------- 构件 ---------------- */

function dockletPiece() {
  const group = new THREE.Group();
  const size = 0.62;
  const gap = 0.14;
  const specs = [
    { glyph: "clock", from: "#8fb4f2", to: "#3d63b4" },
    { glyph: "cpu", from: "#86d7b4", to: "#2f8a68" },
    { glyph: "battery", from: "#ffb27a", to: "#d9603f" },
  ];

  specs.forEach((spec, i) => {
    const x = (i - 1) * (size + gap);
    const tile = createTile({ size, depth: 0.1, from: spec.from, to: spec.to, glyph: spec.glyph });
    tile.group.position.set(x, 0, 0);
    group.add(tile.group);

    if (i === 0) {
      const label = new THREE.Mesh(
        new THREE.PlaneGeometry(size * 0.74, size * 0.26),
        new THREE.MeshBasicMaterial({ map: labelTexture("21:07"), transparent: true })
      );
      label.position.set(x, -size * 0.16, 0.07);
      group.add(label);
    }

    if (i === 1) {
      const ring = new THREE.Mesh(ringGeo(size * 0.4, 0.026), accentMat());
      ring.position.set(x, 0, 0.06);
      group.add(ring);
    }

    if (i === 2) {
      const badge = new THREE.Sprite(
        new THREE.SpriteMaterial({ map: badgeTexture("3"), transparent: true })
      );
      badge.scale.set(0.18, 0.18, 1);
      badge.position.set(x + size * 0.38, size * 0.38, 0.12);
      group.add(badge);
    }
  });

  return group;
}

function menuPiece() {
  const group = new THREE.Group();
  const W = 2.2;
  const H = 2.7;
  const D = 0.14;

  group.add(new THREE.Mesh(panelGeo(W, H, D, 0.1), panelSurface({ color: 0x1b202d })));

  for (let i = 0; i < 5; i++) {
    const y = H / 2 - 0.52 - i * 0.4;
    const row = new THREE.Mesh(
      new THREE.BoxGeometry(W - 0.42, 0.27, 0.03),
      i === 1 ? accentMat(0x35507f, 0x1d3054) : flatMat(0x262c3b)
    );
    row.position.set(0, y, D / 2 + 0.02);
    group.add(row);
  }

  const sep = new THREE.Mesh(new THREE.BoxGeometry(W - 0.54, 0.012, 0.03), flatMat(0x475070));
  sep.position.set(0, H / 2 - 1.42, D / 2 + 0.03);
  group.add(sep);

  return group;
}

function settingsPiece() {
  const group = new THREE.Group();
  const W = 3.3;
  const H = 2.35;
  const D = 0.14;

  group.add(new THREE.Mesh(panelGeo(W, H, D, 0.12), panelSurface({ color: 0x141826 })));

  for (let i = 0; i < 4; i++) {
    const tab = new THREE.Mesh(
      new THREE.BoxGeometry(0.62, 0.2, 0.03),
      i === 1 ? accentMat(0x3b5280, 0x1f3355) : flatMat(0x232a3a)
    );
    tab.position.set(-1.05 + i * 0.7, H / 2 - 0.34, D / 2 + 0.02);
    group.add(tab);
  }

  for (let i = 0; i < 3; i++) {
    const y = 0.32 - i * 0.56;
    const track = new THREE.Mesh(new THREE.BoxGeometry(2.1, 0.05, 0.03), flatMat(0x2a3143));
    track.position.set(-0.12, y, D / 2 + 0.02);
    group.add(track);

    const fillW = 1.0 + i * 0.28;
    const fill = new THREE.Mesh(new THREE.BoxGeometry(fillW, 0.052, 0.032), accentMat());
    fill.position.set(-1.17 + fillW / 2, y, D / 2 + 0.026);
    group.add(fill);

    const knob = new THREE.Mesh(new THREE.CylinderGeometry(0.07, 0.07, 0.04, 20), flatMat(0xe9edf7));
    knob.rotation.x = Math.PI / 2;
    knob.position.set(-1.17 + fillW, y, D / 2 + 0.04);
    group.add(knob);
  }

  const swatch = new THREE.Mesh(new THREE.BoxGeometry(0.26, 0.18, 0.05), flatMat(0x161a26));
  swatch.position.set(1.18, -1.0, D / 2 + 0.03);
  group.add(swatch);

  return group;
}

function markPiece() {
  const group = new THREE.Group();

  const body = new THREE.Mesh(
    squircleGeo(1.15, 0.16, 0.02),
    panelSurface({ color: 0x141a27, roughness: 0.45, metalness: 0.12 })
  );
  group.add(body);

  const shelf = new THREE.Mesh(stadiumGeo(0.78, 0.2, 0.1, 0.02), flatMat(0x2a3242));
  shelf.position.set(0, -0.25, 0.14);
  group.add(shelf);

  const tile = createTile({ size: 0.3, depth: 0.08, from: "#9fb8ee", to: "#5a76bd", glyph: "dot" });
  tile.group.position.set(0, 0.06, 0.14);
  group.add(tile.group);

  return group;
}

/* ---------------- 布景 ---------------- */

const showroom = new THREE.Group();
scene.add(showroom);

const dock = buildDock();
dock.position.set(0, 0, 1.3);
showroom.add(dock);

const docklets = dockletPiece();
docklets.position.set(-0.1, 1.62, 0.35);
docklets.rotation.set(-0.12, 0.1, 0.02);
showroom.add(docklets);

const menu = menuPiece();
menu.position.set(-3.95, 0.95, -0.05);
menu.rotation.set(-0.04, 0.36, 0.02);
showroom.add(menu);

const settings = settingsPiece();
settings.position.set(4.0, 1.0, -0.3);
settings.rotation.set(-0.04, -0.36, -0.02);
showroom.add(settings);

const mark = markPiece();
mark.position.set(0, 3.25, -1.9);
mark.rotation.set(-0.05, 0.12, 0);
showroom.add(mark);

const labels = [
  ["Dock", 0, -0.72, 1.3, 1.5],
  ["Docklets", 0, 2.12, 0.35, 1.9],
  ["Context menu", -3.95, -0.8, -0.05, 2.3],
  ["Settings", 4.0, -0.58, -0.3, 1.9],
  ["Mark", 0, 4.05, -1.9, 1.3],
];
for (const [text, x, y, z, w] of labels) {
  const sprite = textSprite(text, w);
  sprite.position.set(x, y, z);
  showroom.add(sprite);
}

/* ---------------- 取景与循环 ---------------- */

const dir = new THREE.Vector3(0.3, 0.22, 1).normalize();
let lastAspect = 0;

function frame() {
  const w = canvas.clientWidth || 1;
  const h = canvas.clientHeight || 1;
  renderer.setSize(w, h, false);
  camera.aspect = w / h;

  if (Math.abs(camera.aspect - lastAspect) > 0.2 || lastAspect === 0) {
    const distance = Math.max(15, 13.5 * (1.65 / Math.max(camera.aspect, 0.7)));
    camera.position.copy(controls.target).add(dir.clone().multiplyScalar(distance));
    controls.update();
    lastAspect = camera.aspect;
  }

  camera.updateProjectionMatrix();
}

window.addEventListener("resize", frame);

const clock = new THREE.Clock();
let raf = 0;

function tick() {
  const t = clock.getElapsedTime();
  if (!reduceMotion) {
    showroom.position.y = Math.sin(t * 0.5) * 0.03;
    docklets.position.y = 1.62 + Math.sin(t * 0.8) * 0.05;
    mark.rotation.z = Math.sin(t * 0.4) * 0.012;
  }
  controls.update();
  renderer.render(scene, camera);
  raf = requestAnimationFrame(tick);
}

document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    cancelAnimationFrame(raf);
    raf = 0;
  } else if (!raf) {
    clock.getDelta();
    raf = requestAnimationFrame(tick);
  }
});

const rotateBtn = document.getElementById("rotate");
rotateBtn.setAttribute("aria-pressed", String(controls.autoRotate));
rotateBtn.addEventListener("click", () => {
  controls.autoRotate = !controls.autoRotate;
  rotateBtn.setAttribute("aria-pressed", String(controls.autoRotate));
});

document.getElementById("reset").addEventListener("click", () => {
  lastAspect = 0;
  frame();
});

frame();
raf = requestAnimationFrame(tick);
