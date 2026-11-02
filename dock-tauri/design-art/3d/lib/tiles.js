/* 图标磁贴：Dock、插件磁贴、展厅都复用这一个构件。 */

import * as THREE from "../../vendor/three/three.module.js";
import { squircleGeo } from "./geometry.js";
import { tileEdge, tileFace } from "./materials.js";
import { iconTexture } from "./textures.js";

export function createTile(opts = {}) {
  const size = opts.size ?? 0.56;
  const depth = opts.depth ?? 0.1;
  const bevel = opts.bevel ?? 0.012;

  const geo = squircleGeo(size, depth, bevel);
  const texture = iconTexture({
    from: opts.from ?? "#5b78c4",
    to: opts.to ?? "#334a7d",
    glyph: opts.glyph ?? "dot",
    fg: opts.fg,
  });

  const face = tileFace(texture, size);
  const edge = tileEdge(opts.to ?? "#334a7d");
  const mesh = new THREE.Mesh(geo, [face, edge]);

  const group = new THREE.Group();
  group.add(mesh);

  return { group, mesh, face, edge, geometry: geo, texture };
}
