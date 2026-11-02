/* Dock 三维渲染层
   把 Dock 的药丸、图标、运行点、角标画成真三维，交互沿用产品参数：
   峰值 1.6、衰减 0.45、作用半径 2.2 格、底边对齐放大。
   几何、材质、贴图全部来自 lib/，与展厅共用同一套构件。 */

import * as THREE from "../vendor/three/three.module.js";
import { RoomEnvironment } from "../vendor/three/addons/environments/RoomEnvironment.js";
import { stadiumGeo } from "./lib/geometry.js";
import { glass } from "./lib/materials.js";
import { badgeTexture } from "./lib/textures.js";
import { createTile } from "./lib/tiles.js";

const FALLOFF = 0.45;

export function createDock3D(canvas, options = {}) {
  const opts = Object.assign(
    {
      iconSize: 0.56,
      gap: 0.09,
      padH: 0.2,
      padV: 0.11,
      depth: 0.1,
      peak: 1.6,
      radiusIcons: 2.2,
      depthLift: 0.16,
      items: [],
      onHover: null,
      onActivate: null,
    },
    options
  );

  const bar = opts.iconSize + opts.padV * 2;
  const pitch = opts.iconSize + opts.gap;
  const TILE_Z = 0.18;

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.06;

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(24, 1, 0.1, 60);

  const pmrem = new THREE.PMREMGenerator(renderer);
  scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
  pmrem.dispose();

  const key = new THREE.DirectionalLight(0xe8f0ff, 2.6);
  key.position.set(-1.6, 3.2, 3.4);
  scene.add(key);

  const rim = new THREE.DirectionalLight(0x7c9be0, 1.9);
  rim.position.set(2.4, 1.2, -2.2);
  scene.add(rim);

  scene.add(new THREE.HemisphereLight(0xb4c8ff, 0x080b12, 0.5));

  const root = new THREE.Group();
  scene.add(root);

  let items = [];
  let width = 0;

  function clear() {
    while (root.children.length) {
      const child = root.children[0];
      root.remove(child);
      child.traverse((node) => {
        if (node.geometry) node.geometry.dispose();
        if (node.material) {
          const mats = Array.isArray(node.material) ? node.material : [node.material];
          mats.forEach((m) => {
            if (m.map) m.map.dispose();
            m.dispose();
          });
        }
      });
    }
  }

  function build() {
    clear();

    const slots = [];
    let x = 0;
    for (const item of items) {
      if (item.kind === "sep") {
        slots.push({ item, x: x + 0.03 });
        x += 0.06;
      } else {
        slots.push({ item, x: x + opts.iconSize / 2 });
        x += opts.iconSize + opts.gap;
      }
    }
    width = Math.max(x - opts.gap, 0.1);
    for (const slot of slots) slot.x -= width / 2;

    const pill = new THREE.Mesh(stadiumGeo(width + opts.padH * 2, bar, 0.22, 0.03), glass());
    root.add(pill);

    for (const slot of slots) {
      if (slot.item.kind === "sep") {
        const sep = new THREE.Mesh(
          new THREE.BoxGeometry(0.012, bar * 0.56, 0.02),
          new THREE.MeshStandardMaterial({
            color: 0xdfe6f5,
            transparent: true,
            opacity: 0.18,
            roughness: 0.4,
          })
        );
        sep.position.set(slot.x, 0, 0.02);
        root.add(sep);
        continue;
      }

      const tile = createTile({
        size: opts.iconSize,
        depth: opts.depth,
        from: slot.item.from,
        to: slot.item.to,
        glyph: slot.item.glyph,
      });
      tile.group.position.set(slot.x, 0, TILE_Z);
      tile.group.userData = { baseX: slot.x, item: slot.item };
      root.add(tile.group);

      if (slot.item.badge) {
        const sprite = new THREE.Sprite(
          new THREE.SpriteMaterial({ map: badgeTexture(slot.item.badge), transparent: true })
        );
        sprite.scale.set(0.15, 0.15, 1);
        sprite.position.set(
          opts.iconSize * 0.36,
          opts.iconSize * 0.36,
          opts.depth / 2 + 0.06
        );
        tile.group.add(sprite);
      }

      // 运行点挂在 Dock 上，图标放大时它留在原位
      if (slot.item.running) {
        const dot = new THREE.Mesh(
          new THREE.SphereGeometry(0.024, 16, 12),
          new THREE.MeshStandardMaterial({
            color: 0xffffff,
            emissive: 0xffffff,
            emissiveIntensity: 0.9,
            roughness: 0.4,
          })
        );
        dot.position.set(slot.x, -bar / 2 + 0.075, 0.26);
        root.add(dot);
      }
    }
  }

  /* ---------------- 交互 ---------------- */

  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();
  const hoverPlane = new THREE.Plane(new THREE.Vector3(0, 0, 1), TILE_Z);
  const hit = new THREE.Vector3();

  let pointerX = null;
  let hoverIndex = -1;
  let pressedIndex = -1;
  let depthScale = 1;

  function tiles() {
    return root.children.filter((c) => c.userData && c.userData.item);
  }

  function updatePointer(px, py) {
    const rect = canvas.getBoundingClientRect();
    pointer.x = (px / rect.width) * 2 - 1;
    pointer.y = -(py / rect.height) * 2 + 1;
    raycaster.setFromCamera(pointer, camera);
    pointerX = raycaster.ray.intersectPlane(hoverPlane, hit) ? hit.x : null;
  }

  // 供键盘导航或外部状态驱动：把放大中心钉在某枚图标上
  function focus(index) {
    const list = tiles();
    pointerX =
      index === null || index === undefined || index < 0 || index >= list.length
        ? null
        : list[index].userData.baseX;
  }

  canvas.addEventListener("pointermove", (e) => {
    const rect = canvas.getBoundingClientRect();
    updatePointer(e.clientX - rect.left, e.clientY - rect.top);
  });
  canvas.addEventListener("pointerleave", () => {
    pointerX = null;
  });
  canvas.addEventListener("pointerdown", () => {
    pressedIndex = hoverIndex;
  });
  canvas.addEventListener("pointerup", () => {
    if (pressedIndex >= 0 && pressedIndex === hoverIndex && opts.onActivate) {
      opts.onActivate(hoverIndex, tiles()[hoverIndex].userData.item);
    }
    pressedIndex = -1;
  });
  canvas.style.touchAction = "none";

  /* ---------------- 取景 ---------------- */

  function frame() {
    const w = canvas.clientWidth || 1;
    const h = canvas.clientHeight || 1;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;

    const needWidth = (width + opts.padH * 2) * 1.16;
    const needHeight = bar * 2.05;
    const halfFov = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
    const distance = Math.max(
      needWidth / camera.aspect / (2 * halfFov),
      needHeight / (2 * halfFov),
      1.6
    );

    const visibleHeight = 2 * halfFov * distance;
    const targetY = visibleHeight * 0.18;
    camera.position.set(0, targetY + distance * 0.07, distance);
    camera.lookAt(0, targetY, 0);
    camera.updateProjectionMatrix();
  }

  /* ---------------- 循环 ---------------- */

  const clock = new THREE.Clock();
  const projected = new THREE.Vector3();
  let raf = 0;

  function tick() {
    const dt = Math.min(clock.getDelta(), 0.05);
    const list = tiles();
    let nextHover = -1;
    let hoverScreenX = 0;
    let hoverScreenY = 0;

    for (let i = 0; i < list.length; i++) {
      const tile = list[i];
      let scale = 1;

      if (pointerX !== null) {
        const d = Math.abs(tile.userData.baseX - pointerX) / pitch;
        const weight =
          d >= opts.radiusIcons ? 0 : Math.exp(-Math.pow(d / opts.radiusIcons / FALLOFF, 4));
        scale = 1 + (opts.peak - 1) * weight;
        if (weight > 0.55) nextHover = i;
      }

      const pressed = pressedIndex === i && hoverIndex === i ? 0.94 : 1;
      const target = scale * pressed;
      const next = tile.scale.x + (target - tile.scale.x) * Math.min(1, dt * 14);

      tile.scale.setScalar(next);
      tile.position.y = ((next - 1) * opts.iconSize) / 2;
      tile.position.z = TILE_Z + ((next - 1) / (opts.peak - 1)) * opts.depthLift * depthScale;

      if (i === nextHover) {
        tile.getWorldPosition(projected);
        projected.project(camera);
        hoverScreenX = (projected.x * 0.5 + 0.5) * canvas.clientWidth;
        hoverScreenY = (-projected.y * 0.5 + 0.5) * canvas.clientHeight;
      }
    }

    if (nextHover !== hoverIndex) {
      hoverIndex = nextHover;
      if (opts.onHover) {
        opts.onHover(
          hoverIndex,
          hoverScreenX,
          hoverScreenY,
          hoverIndex >= 0 ? list[hoverIndex].userData.item : null
        );
      }
    } else if (hoverIndex >= 0 && opts.onHover) {
      opts.onHover(hoverIndex, hoverScreenX, hoverScreenY, list[hoverIndex].userData.item);
    }

    renderer.render(scene, camera);
    raf = requestAnimationFrame(tick);
  }

  /* ---------------- 生命周期 ---------------- */

  function setItems(next) {
    items = next;
    build();
    frame();
  }

  function setDepth(value) {
    depthScale = Math.max(0, Math.min(1, value));
  }

  const observer = new ResizeObserver(frame);
  observer.observe(canvas);

  setItems(opts.items);
  raf = requestAnimationFrame(tick);

  return {
    setItems,
    setDepth,
    focus,
    resize: frame,
    dispose() {
      cancelAnimationFrame(raf);
      observer.disconnect();
      clear();
      renderer.dispose();
    },
    get hoverIndex() {
      return hoverIndex;
    },
  };
}
