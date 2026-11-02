# Dock 三维层

这里有两个入口，用途不同：

| 文件 | 用途 |
|---|---|
| `showroom.html` | **整套美术资源的三维展厅**：Dock、插件磁贴、菜单、设置面板、品牌标记 |
| `styles.html` | **三种风格的三维对照**：Glass、Bare、Frost 用同一套图标 |
| `dock-3d.html` | **能放进 Dock 的三维渲染层**，与产品同形 |
| `index.html` | 三维产品视觉稿，用来研究厚度、高光与悬浮感 |

```
3d/
  lib/geometry.js    超椭圆、药丸、圆角面板
  lib/materials.js   玻璃、图标面、面板、强调
  lib/textures.js    图标正面、角标、文字药丸
  lib/tiles.js       图标磁贴构件
  lib/dock-piece.js  静态 Dock 构件，支持三种风格
  lib/labels.js      三维文字标签
  lib/backdrop.js    背景与地面
  showroom.js        展厅布景
  styles.js          风格对照布景
  dock-renderer.js   可挂载的 Dock 层
```

双击即可打开，`dock-demo.bundle.js` 与 `dock3d.bundle.js` 已把 three.js 打进单文件，
不需要服务器，也不需要联网。

## 三维层：dock-3d.html

药丸、图标、分隔线、运行点、角标全部由 WebGL 绘制，布局与产品一致：

- 图标 0.56 单位，间距 0.09，内边距 0.2 / 0.11，药丸高度 0.78
- 图标用 22.37% 超椭圆挤出，正面贴 Canvas 生成的图标纹理，侧面是深一档的颜色
- 药丸按实际宽度重建几何，圆角始终等于高度 / 2，不用 `scale.x` 拉伸
- 悬停沿用药丸的放大曲线：峰值 1.6、衰减 0.45、半径 2.2 格
- 放大时底边对齐，并向观察者抬起，邻接图标按同一权重递减
- 运行点挂在 Dock 上，图标放大时它留在原位
- 角标挂在图标上，随图标一起放大

### 接入方式

```js
import { createDock3D } from "./dock-renderer.js";

const dock = createDock3D(canvas, {
  items: [
    { kind: "app", glyph: "ring", from: "#ffb27a", to: "#d9603f", name: "Slate", running: true },
    { kind: "sep" },
    { kind: "app", glyph: "dot", from: "#eba7a7", to: "#a34c4c", name: "Prism", badge: "3" },
  ],
  onHover(index, x, y, item) {
    // DOM 气泡用返回的屏幕坐标定位，文字始终留在 DOM
  },
  onActivate(index, item) {
    // 点击或回车
  },
});

dock.setItems(nextItems);   // 增删图标
dock.setDepth(0.6);         // 0 到 1，控制 Z 抬升
dock.focus(3);              // 键盘导航：把放大中心钉在第 4 枚图标
dock.resize();              // 容器尺寸变化
dock.dispose();             // 卸载
```

图标条目字段：

| 字段 | 说明 |
|---|---|
| `kind` | `app`、`sys`、`sep` |
| `glyph` | `ring` `bars` `triangle` `dot` `cross` `grid` `sliders` `folder` `trash` |
| `from` / `to` | 图标正面的渐变起止色 |
| `name` | 气泡文案 |
| `running` | 是否显示运行点 |
| `badge` | 角标文字，省略则不显示 |

### 与产品的边界

三维层只负责 Dock 这一条。以下仍然留在 DOM：

- 右键菜单、设置面板、提示气泡的文字
- 插件磁贴里的数字（等宽字体与本地化不该进 WebGL）
- 拖拽排序的命中判定

接入前要确认两件事：

1. 透明置顶窗口里增加一个 WebGL 画布会多一个合成层，笔记本双显卡切换时可能掉帧，
   需要在目标机器上实测空闲占用。
2. 没有 WebGL 时回退到现有的 DOM Dock，不要显示空白。

## 展厅：showroom.html

五个构件用同一套 `lib/` 立在一个场景里，各自带三维标签，可拖动环绕：

- Dock 主栏：药丸加五枚图标，含运行点与角标
- 插件磁贴：时钟的数值药丸、处理器的圆环、电池的角标
- 右键菜单：五行条目，一行强调态，一条分隔线
- 设置面板：四个标签、三条滑杆、一个色块
- 品牌标记：超椭圆本体、玻璃托架、抬升磁贴

## 视觉稿：index.html

独立的展示场景，用来检查材质：可拖动旋转、自动旋转开关、重置视角。
参数与三维层一致，玻璃是半透明加清漆，不用 `transmission`。

## 重新构建

修改 `scene.js` 或 `dock-renderer.js` / `dock-demo.js` 后都要重新打包：

```
npx esbuild 3d/dock-demo.js --bundle --format=iife --minify --target=chrome110 \
  --alias:three=./vendor/three/three.module.js --outfile=3d/dock-demo.bundle.js

npx esbuild 3d/scene.js --bundle --format=iife --minify --target=chrome110 \
  --alias:three=./vendor/three/three.module.js --outfile=3d/dock3d.bundle.js
```

在 `design-art/` 目录下执行。打成 IIFE 是为了 `file://` 直接打开也能跑，
ES module 在 `file://` 下会被 CORS 拦掉。

## 依赖

`../vendor/three/` 是 three.js r160 的本地副本，含 OrbitControls 与 RoomEnvironment，
许可见 `../vendor/three/LICENSE`（MIT）。不引入其他依赖。
