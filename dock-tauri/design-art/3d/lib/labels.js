/* 三维文字标签：始终面向相机，用于给展厅里的构件命名。 */

import * as THREE from "../../vendor/three/three.module.js";
import { labelTexture } from "./textures.js";

export function textSprite(text, width = 1.9, color = "#aab4c8") {
  const ratio = 120 / 640;
  const tex = labelTexture(text, {
    width: 640,
    height: 120,
    pill: false,
    color,
    size: 46,
    weight: 500,
  });
  const sprite = new THREE.Sprite(
    new THREE.SpriteMaterial({
      map: tex,
      transparent: true,
      depthTest: false,
      depthWrite: false,
    })
  );
  sprite.scale.set(width, width * ratio, 1);
  sprite.renderOrder = 10;
  return sprite;
}
