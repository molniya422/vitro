/* 三种风格的三维对照：Glass、Bare、Frost 用同一套图标，只改药丸、间距与光环。 */

import * as THREE from "../vendor/three/three.module.js";
import { OrbitControls } from "../vendor/three/addons/controls/OrbitControls.js";
import { RoomEnvironment } from "../vendor/three/addons/environments/RoomEnvironment.js";
import { backdropTexture, createFloor } from "./lib/backdrop.js";
import { buildDock } from "./lib/dock-piece.js";
import { textSprite } from "./lib/labels.js";

const canvas = document.getElementById("scene");
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.08;

const scene = new THREE.Scene();
scene.background = backdropTexture();
scene.add(createFloor(-2.95));

const camera = new THREE.PerspectiveCamera(30, 1, 0.1, 80);
const controls = new OrbitControls(camera, canvas);
controls.target.set(0, 0, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.07;
controls.enablePan = false;
controls.minDistance = 6;
controls.maxDistance = 22;
controls.minPolarAngle = THREE.MathUtils.degToRad(45);
controls.maxPolarAngle = THREE.MathUtils.degToRad(85);
controls.autoRotate = !reduceMotion;
controls.autoRotateSpeed = 0.4;

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

const rows = [
  { style: "glass", label: "Glass", y: 1.9 },
  { style: "bare", label: "Bare", y: 0 },
  { style: "frost", label: "Frost", y: -1.9 },
];

const docks = [];

for (const row of rows) {
  const dock = buildDock({ style: row.style, size: 0.62, bar: 0.82 });
  dock.position.set(0, row.y, 0);
  scene.add(dock);
  docks.push({ group: dock, baseY: row.y });

  const label = textSprite(row.label, 1.7);
  label.position.set(2.45, row.y, 0);
  scene.add(label);
}

/* ---------------- 取景与循环 ---------------- */

const dir = new THREE.Vector3(0.32, 0.16, 1).normalize();
let lastAspect = 0;

function frame() {
  const w = canvas.clientWidth || 1;
  const h = canvas.clientHeight || 1;
  renderer.setSize(w, h, false);
  camera.aspect = w / h;

  if (Math.abs(camera.aspect - lastAspect) > 0.2 || lastAspect === 0) {
    const distance = Math.max(11.8, 10.8 * (1.65 / Math.max(camera.aspect, 0.7)));
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
    for (let i = 0; i < docks.length; i++) {
      docks[i].group.position.y = docks[i].baseY + Math.sin(t * 0.7 + i * 0.9) * 0.035;
    }
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
