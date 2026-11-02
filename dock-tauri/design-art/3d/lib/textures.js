/* 共享贴图：图标正面、角标、文字药丸。全部用 Canvas 现画，不依赖外部图片。 */

import * as THREE from "../../vendor/three/three.module.js";
import { SQUIRCLE_N } from "./geometry.js";

function squirclePath(g, size) {
  const r = size / 2;
  const steps = 96;
  g.beginPath();
  for (let i = 0; i <= steps; i++) {
    const t = (i / steps) * Math.PI * 2;
    const c = Math.cos(t);
    const s = Math.sin(t);
    const x = size / 2 + Math.sign(c) * Math.pow(Math.abs(c), 2 / SQUIRCLE_N) * r;
    const y = size / 2 + Math.sign(s) * Math.pow(Math.abs(s), 2 / SQUIRCLE_N) * r;
    if (i === 0) g.moveTo(x, y);
    else g.lineTo(x, y);
  }
  g.closePath();
}

function glyph(g, kind, S) {
  const c = S / 2;
  g.lineWidth = S * 0.062;
  g.lineCap = "round";
  g.lineJoin = "round";

  if (kind === "ring") {
    g.beginPath();
    g.arc(c, c, S * 0.19, 0, Math.PI * 2);
    g.stroke();
  } else if (kind === "bars") {
    [0.36, 0.5, 0.64].forEach((p, i) => {
      const w = S * (0.46 - i * 0.09);
      g.beginPath();
      g.roundRect(c - w / 2, S * p - S * 0.035, w, S * 0.07, S * 0.035);
      g.fill();
    });
  } else if (kind === "triangle") {
    g.beginPath();
    g.moveTo(c, c - S * 0.19);
    g.lineTo(c + S * 0.2, c + S * 0.17);
    g.lineTo(c - S * 0.2, c + S * 0.17);
    g.closePath();
    g.fill();
  } else if (kind === "dot") {
    g.beginPath();
    g.arc(c, c, S * 0.15, 0, Math.PI * 2);
    g.fill();
  } else if (kind === "cross") {
    g.beginPath();
    g.roundRect(c - S * 0.2, c - S * 0.035, S * 0.4, S * 0.07, S * 0.035);
    g.roundRect(c - S * 0.035, c - S * 0.2, S * 0.07, S * 0.4, S * 0.035);
    g.fill();
  } else if (kind === "grid") {
    const cell = S * 0.17;
    const gap = S * 0.06;
    [-1, 1].forEach((sx) => {
      [-1, 1].forEach((sy) => {
        g.beginPath();
        g.roundRect(
          c + sx * (cell / 2 + gap / 2) - cell / 2,
          c + sy * (cell / 2 + gap / 2) - cell / 2,
          cell,
          cell,
          S * 0.04
        );
        g.fill();
      });
    });
  } else if (kind === "sliders") {
    const y1 = S * 0.4;
    const y2 = S * 0.6;
    g.beginPath();
    g.moveTo(S * 0.26, y1);
    g.lineTo(S * 0.58, y1);
    g.moveTo(S * 0.26, y2);
    g.lineTo(S * 0.42, y2);
    g.moveTo(S * 0.56, y2);
    g.lineTo(S * 0.74, y2);
    g.stroke();
    g.beginPath();
    g.arc(S * 0.68, y1, S * 0.055, 0, Math.PI * 2);
    g.arc(S * 0.48, y2, S * 0.055, 0, Math.PI * 2);
    g.fill();
  } else if (kind === "folder") {
    g.beginPath();
    g.roundRect(S * 0.25, S * 0.36, S * 0.5, S * 0.32, S * 0.05);
    g.stroke();
    g.beginPath();
    g.moveTo(S * 0.25, S * 0.4);
    g.lineTo(S * 0.25, S * 0.32);
    g.quadraticCurveTo(S * 0.25, S * 0.28, S * 0.3, S * 0.28);
    g.lineTo(S * 0.42, S * 0.28);
    g.lineTo(S * 0.47, S * 0.36);
    g.stroke();
  } else if (kind === "trash") {
    g.beginPath();
    g.moveTo(S * 0.3, S * 0.36);
    g.lineTo(S * 0.7, S * 0.36);
    g.stroke();
    g.beginPath();
    g.roundRect(S * 0.34, S * 0.4, S * 0.32, S * 0.26, S * 0.04);
    g.stroke();
    g.beginPath();
    g.moveTo(S * 0.45, S * 0.36);
    g.lineTo(S * 0.45, S * 0.31);
    g.lineTo(S * 0.55, S * 0.31);
    g.lineTo(S * 0.55, S * 0.36);
    g.stroke();
  } else if (kind === "clock") {
    g.beginPath();
    g.arc(c, c, S * 0.2, 0, Math.PI * 2);
    g.stroke();
    g.beginPath();
    g.moveTo(c, c - S * 0.11);
    g.lineTo(c, c);
    g.lineTo(c + S * 0.1, c + S * 0.05);
    g.stroke();
  } else if (kind === "cpu") {
    g.beginPath();
    g.roundRect(c - S * 0.13, c - S * 0.13, S * 0.26, S * 0.26, S * 0.05);
    g.stroke();
    g.beginPath();
    [-0.09, 0, 0.09].forEach((p) => {
      g.moveTo(c + p * S, c - S * 0.2);
      g.lineTo(c + p * S, c - S * 0.13);
      g.moveTo(c + p * S, c + S * 0.13);
      g.lineTo(c + p * S, c + S * 0.2);
      g.moveTo(c - S * 0.2, c + p * S);
      g.lineTo(c - S * 0.13, c + p * S);
      g.moveTo(c + S * 0.13, c + p * S);
      g.lineTo(c + S * 0.2, c + p * S);
    });
    g.stroke();
  } else if (kind === "battery") {
    g.beginPath();
    g.roundRect(c - S * 0.2, c - S * 0.1, S * 0.4, S * 0.2, S * 0.05);
    g.stroke();
    g.beginPath();
    g.roundRect(c - S * 0.13, c - S * 0.05, S * 0.08, S * 0.1, S * 0.02);
    g.roundRect(c - S * 0.02, c - S * 0.05, S * 0.08, S * 0.1, S * 0.02);
    g.fill();
  }
}

export function iconTexture(spec) {
  const S = 256;
  const canvas = document.createElement("canvas");
  canvas.width = S;
  canvas.height = S;
  const g = canvas.getContext("2d");

  g.save();
  squirclePath(g, S);
  const grd = g.createLinearGradient(0, 0, 0, S);
  grd.addColorStop(0, spec.from);
  grd.addColorStop(1, spec.to);
  g.fillStyle = grd;
  g.fill();
  const sheen = g.createLinearGradient(0, 0, 0, S * 0.48);
  sheen.addColorStop(0, "rgba(255,255,255,0.24)");
  sheen.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = sheen;
  g.fill();
  g.restore();

  g.fillStyle = spec.fg || "rgba(255,255,255,0.94)";
  g.strokeStyle = spec.fg || "rgba(255,255,255,0.94)";
  glyph(g, spec.glyph, S);

  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  tex.anisotropy = 4;
  return tex;
}

export function badgeTexture(text) {
  const S = 128;
  const canvas = document.createElement("canvas");
  canvas.width = S;
  canvas.height = S;
  const g = canvas.getContext("2d");
  const grd = g.createLinearGradient(0, 0, 0, S);
  grd.addColorStop(0, "#e2605a");
  grd.addColorStop(1, "#c8433d");
  g.fillStyle = grd;
  g.beginPath();
  g.arc(S / 2, S / 2, S / 2 - 2, 0, Math.PI * 2);
  g.fill();
  g.strokeStyle = "rgba(255,255,255,0.45)";
  g.lineWidth = 5;
  g.beginPath();
  g.arc(S / 2, S / 2, S / 2 - 5, Math.PI, 0);
  g.stroke();
  g.fillStyle = "#ffffff";
  g.font = "600 68px Segoe UI, system-ui, sans-serif";
  g.textAlign = "center";
  g.textBaseline = "middle";
  g.fillText(text, S / 2, S / 2 + 3);
  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

/* 文字药丸：docklet 的数值、菜单里的标签都用它，保持文字清晰且不依赖字体打包 */
export function labelTexture(text, opts = {}) {
  const W = opts.width || 256;
  const H = opts.height || 96;
  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const g = canvas.getContext("2d");

  if (opts.pill !== false) {
    g.fillStyle = opts.background || "rgba(8,11,18,0.62)";
    g.beginPath();
    g.roundRect(2, 2, W - 4, H - 4, (H - 4) / 2);
    g.fill();
  }

  g.fillStyle = opts.color || "#ffffff";
  g.font = `${opts.weight || 600} ${opts.size || Math.round(H * 0.52)}px "Cascadia Mono", Consolas, monospace`;
  g.textAlign = "center";
  g.textBaseline = "middle";
  g.fillText(text, W / 2, H / 2 + 1);

  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}
