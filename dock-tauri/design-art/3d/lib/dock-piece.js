/* 静态 Dock 构件：展厅与风格对比页共用。
   三种风格只改药丸、间距与光环，图标本身完全相同。 */

import * as THREE from "../../vendor/three/three.module.js";
import { squircleRingGeo, stadiumGeo } from "./geometry.js";
import { flatMat, frostMaterial, glass } from "./materials.js";
import { badgeTexture } from "./textures.js";
import { createTile } from "./tiles.js";

export const DEFAULT_ITEMS = [
  { glyph: "sliders", from: "#2b3242", to: "#181d29" },
  { glyph: "folder", from: "#ffd479", to: "#d99a2b" },
  { glyph: "ring", from: "#ffb27a", to: "#d9603f", running: true },
  { glyph: "bars", from: "#8fb4f2", to: "#3d63b4", badge: "3" },
];

export function buildDock(opts = {}) {
  const style = opts.style ?? "glass";
  const items = opts.items ?? DEFAULT_ITEMS;
  const size = opts.size ?? 0.52;
  const bar = opts.bar ?? 0.7;
  const gap = style === "bare" ? 0.15 : 0.08;
  const padH = style === "bare" ? 0 : 0.18;
  const tileZ = style === "bare" ? 0 : 0.16;

  const width = items.length * size + (items.length - 1) * gap;
  const group = new THREE.Group();

  if (style !== "bare") {
    const material = style === "frost" ? frostMaterial() : glass();
    group.add(new THREE.Mesh(stadiumGeo(width + padH * 2, bar, 0.2, 0.03), material));

    if (style === "frost") {
      // 1 单位勾边：略大的药丸放在后面，只露出轮廓
      const rim = new THREE.Mesh(
        stadiumGeo(width + padH * 2 + 0.024, bar + 0.024, 0.02, 0.008),
        flatMat(0xdfe8fb, 0.16)
      );
      rim.position.z = -0.09;
      group.add(rim);
    }
  }

  items.forEach((item, i) => {
    const x = -width / 2 + size / 2 + i * (size + gap);

    const tile = createTile({
      size,
      depth: 0.09,
      from: item.from,
      to: item.to,
      glyph: item.glyph,
    });
    tile.group.position.set(x, 0, tileZ);
    group.add(tile.group);

    if (style === "bare") {
      const halo = new THREE.Mesh(
        squircleRingGeo(size + 0.03, 0.008, 0.012),
        new THREE.MeshBasicMaterial({ color: 0xdfe9ff, transparent: true, opacity: 0.55 })
      );
      halo.position.set(x, 0, 0.02);
      group.add(halo);
    }

    if (item.running) {
      const dot = new THREE.Mesh(
        new THREE.SphereGeometry(0.022, 16, 12),
        new THREE.MeshStandardMaterial({
          color: 0xffffff,
          emissive: 0xffffff,
          emissiveIntensity: 0.9,
        })
      );
      dot.position.set(x, -bar / 2 + 0.07, 0.24);
      group.add(dot);
    }

    if (item.badge) {
      const badge = new THREE.Sprite(
        new THREE.SpriteMaterial({ map: badgeTexture(item.badge), transparent: true })
      );
      badge.scale.set(0.15, 0.15, 1);
      badge.position.set(x + size * 0.36, size * 0.36, tileZ + 0.1);
      group.add(badge);
    }
  });

  return group;
}
