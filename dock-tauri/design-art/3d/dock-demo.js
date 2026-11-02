/* Dock 三维层演示：与产品同形的药丸、图标、分隔线、运行点、角标。 */

import { createDock3D } from "./dock-renderer.js";

const canvas = document.getElementById("dock");
const tip = document.getElementById("tip");
const depthInput = document.getElementById("depth");
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const SYS_FROM = "#2b3242";
const SYS_TO = "#181d29";

const ITEMS = [
  { kind: "sys", glyph: "sliders", name: "Dock settings", from: SYS_FROM, to: SYS_TO },
  { kind: "sys", glyph: "grid", name: "Launchpad", from: SYS_FROM, to: SYS_TO },
  { kind: "sep" },
  { kind: "app", glyph: "folder", name: "File Explorer", from: "#ffd479", to: "#d99a2b" },
  { kind: "app", glyph: "ring", name: "Slate", from: "#ffb27a", to: "#d9603f" },
  { kind: "app", glyph: "bars", name: "Orbit", from: "#8fb4f2", to: "#3d63b4", running: true },
  { kind: "app", glyph: "triangle", name: "Fold", from: "#86d7b4", to: "#2f8a68", badge: "3" },
  { kind: "app", glyph: "dot", name: "Prism", from: "#eba7a7", to: "#a34c4c", running: true },
  { kind: "app", glyph: "cross", name: "Ledger", from: "#9aa6bd", to: "#4d5773" },
  { kind: "sep" },
  { kind: "sys", glyph: "trash", name: "Recycle Bin", from: SYS_FROM, to: SYS_TO },
];

const dock = createDock3D(canvas, {
  items: ITEMS,
  depthLift: 0.16,
  onHover(index, x, y, item) {
    if (index < 0 || !item || item.kind === "sep") {
      tip.hidden = true;
      return;
    }
    if (tip.textContent !== item.name) tip.textContent = item.name;
    tip.hidden = false;
    tip.style.left = x + "px";
    tip.style.top = Math.max(6, y - 88) + "px";
  },
});

// 键盘：左右移动放大焦点，Esc 取消。三维层本身不承载可访问名称，文字由气泡给出。
const tileCount = ITEMS.filter((item) => item.kind !== "sep").length;
let focusIndex = -1;

window.addEventListener("keydown", (event) => {
  if (event.key === "ArrowRight") {
    focusIndex = (focusIndex + 1) % tileCount;
    dock.focus(focusIndex);
  } else if (event.key === "ArrowLeft") {
    focusIndex = (focusIndex - 1 + tileCount) % tileCount;
    dock.focus(focusIndex);
  } else if (event.key === "Escape") {
    focusIndex = -1;
    dock.focus(null);
  }
});

// 便于截图与评审：dock-3d.html?hover=4
const initialHover = new URLSearchParams(location.search).get("hover");
if (initialHover !== null) {
  focusIndex = Number(initialHover);
  dock.focus(focusIndex);
}

depthInput.addEventListener("input", () => {
  dock.setDepth(Number(depthInput.value) / 100);
});

if (reduceMotion) {
  dock.setDepth(0);
  depthInput.value = "0";
}

document.getElementById("reset").addEventListener("click", () => {
  depthInput.value = "100";
  dock.setDepth(1);
  dock.resize();
});

window.addEventListener("beforeunload", () => dock.dispose());
