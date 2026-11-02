# Dock 美术资源包 v1

> 目录：`dock-tauri/design-art/`
> 日期：2026-10-26
> 性质：**只读美术资源**。没有修改 `src/`、`src-tauri/`、`docs/` 下的任何文件。
> 预览：双击 `preview/dock.html`（交互）或 `sheet/index.html`（规范页）。无需构建、无需联网。

---

## 0. 一页设计摘要

**视觉命题**：Dock 是一条浮在桌面上的玻璃架，不是一个贴了模糊的矩形。层级由材质表达，不由描边表达；状态写在图标本体上，不新增容器。

**三套风格，全部可选**（用户自由切换，默认 A）：

| 风格 | 组成 | 代价 | 目录 |
|---|---|---|---|
| A 冷调玻璃（默认） | tint 62% + 顶部高光 + 底部内阴影 + 0.25% 颗粒 | 零原生依赖 | `themes/cool-glass/` |
| B 裸排版 | 无底板、无倒影，间距放大到 12，1 px 反色光环保证浅壁纸可读 | 几何会变松 | `themes/bare/` |
| C 磨砂亚克力 | 黑 25% + 模糊 22 + 饱和 1.08 + 1 px 勾边 | 必须 pill 形状原生副窗，否则降级为 A | `themes/frost/` |

**色彩**：一条冷灰阶（`#0f131c` → `#f4f6fb`），一个强调色 `#7c9be0`（221°，饱和 62%）。语义色 `#6fa896 / #d9a45c / #d46a62` 只用于阈值与角标，不参与品牌表达。无机紫、无霓虹、无纯黑、无常驻外发光。

**字体**：系统 `Segoe UI Variable`（标题 Display / 正文 Text），数字 `Cascadia Mono`。不打包字体。

**八个签名数值**：图标 48 px · 间距 8 px · 内边距 16/10 · 药丸圆角 34（= 高/2）· 放大 1.6× · 衰减 0.45 · 运行点 4 px · 超椭圆 22.37%。

**与现有实现的关系**：所有几何数值取自当前代码（`appearance.rs`、`layout.ts`、`magnify.ts`、`dock.css`），所以资源包不是另一套设计，而是同一套数值的资产化。接入顺序见 §7。

---

## 1. 假设

信息不足处按下面处理，不阻塞设计：

1. 产品工作名仍为 Dock；`art/mark.svg` 是占位标记，改名不影响任何其他资产。
2. 目标 Windows 10 21H2+ 与 Windows 11，WebView2 常青版可用。
3. 单窗口四路由（dock / launchpad / monitor / settings）保持现状。
4. Dock 上限 10 项；本包不改变上限，也不设计溢出方案。
5. 首版只实现底部停靠。左右边缘只有美术规格，**行为未实现前不要放出设置项**。
6. 浅色主题目前未实现（代码里只有深色变量）。本包给出完整 token 与两套组件样张，落地需要改代码。
7. 三套风格全部随包发布并可在设置页切换。C（亚克力）在裁不出药丸形模糊区时自动降级为 A。
8. 不引入任何 npm 依赖。所有资产是 SVG / CSS / 原生 JS。
9. 图标来源仍是系统 Shell 提取（`icon_cache.rs`）；本包不提供第三方应用图标，避免授权问题。
10. 现有 6 个字形（folder / gear / grid / chart / sliders / unknown）保持兼容；新字形沿用 24 网格、1.7 描边。

---

## 2. 架构与组件（美术映射）

### 2.1 页面与面板清单

| 面板 | 路由 | 材质 | 玻璃 | 主要资产 |
|---|---|---|---|---|
| Dock 主栏 | `dock` | Glass / Bare / Frost | A 与 C 是，B 否 | 药丸、磁贴、运行点、角标、倒影 |
| Launchpad | `launchpad` | Painted | 是 | 字形、搜索框、应用网格 |
| 系统监控 | `monitor` | Painted 82% | 是（低透明） | 圆环、火花线、数值药丸 |
| 设置 | `settings` | Solid | 否 | 分段控件、滑杆、色块、预览 |
| 右键菜单 | overlay | Solid 97% + blur | 是 | 菜单项、分隔线、危险项 |
| 提示气泡 | overlay | Solid 95% | 是 | 单行标签 |

规则：**设置页不使用玻璃**。控件背后可能有 App 内容，透明会把表单变成猜字游戏。

### 2.2 Dock 组件树

```
DockStage                        .dock-stage，透明窗口舞台
└─ DockPill                      .dock-pill，玻璃四层（tokens/material.css）
   ├─ ControlTile                g-sliders，单击设置 / 双击折叠
   ├─ RouteTile[]                g-grid（Launchpad）、g-chart（监控）
   ├─ Separator[]                art/shapes.svg#separator
   ├─ DockItem[]                 kind: app | folder | plugin | system | trash
   │  ├─ TileArt                 Shell 图标 / g-* 字形 / 插件磁贴
   │  ├─ Reflection              art/shapes.svg#reflection-fade
   │  ├─ RunningDot              art/shapes.svg#running-dot
   │  ├─ Badge                   art/shapes.svg#badge
   │  └─ Tooltip                 单行气泡
   └─ ContextMenu                overlay
```

### 2.3 Item 类型抽象

| kind | 图标来源 | 倒影 | 运行点 | 角标 | 可拖拽 | 可拖出 |
|---|---|---|---|---|---|---|
| `app` | Shell 提取 | 是 | 是 | 最小化数 | 是 | 是 |
| `folder` | `g-folder` | 是 | 否 | 条目数 | 是 | 是 |
| `plugin` | `g-*` + 状态件 | 是 | 否 | 数值 | 是 | 是 |
| `system` | `g-*` | 是 | 否 | 否 | 否 | 否 |
| `trash` | `g-trash` / `g-trash-full` | 是 | 否 | 否 | 否 | 否 |
| `separator` | 1 px 线 | 否 | 否 | 否 | 否 | 否 |

`plugin` 的磁贴槽位规则（与 `art/docklet-tiles.svg` 一致）：glyph、ring、pill、badge、dots 五个槽位，**ring 与 pill 互斥**，badge 与 dots 可叠加。一个磁贴只回答一个问题。

### 2.4 尺寸与状态矩阵

| 项 | 默认 | 范围 | 依据 |
|---|---:|---|---|
| icon size | 48 | 32-80 | `settings.rs` |
| icon gap | 8 | 0-24 | 同上 |
| padding h / v | 16 / 10 | 4-32 / 2-24 | 同上 |
| pill radius | 34 | 0-64 | 默认 = 高 / 2 |
| reflection | 40 px 带 | - | `reflectionSpace` |
| running dot | 4 px | - | macOS 规格 |
| badge | 16 px | - | macOS 规格 |
| separator | 1 × 72% | - | 本包新增 |

---

## 3. 交互与动效

### 3.1 磁贴状态机（美术层）

| 当前 | 事件 | 下一 | 视觉 |
|---|---|---|---|
| Idle | pointerenter | Hover | 放大跟随 160 ms |
| Hover | 停留 500 ms | Hover + Tooltip | 气泡 150 ms 淡入，位移 6 → 0 |
| Hover | pointerdown | Pressed | 内部图像 `scale(0.9)`，90 ms |
| Pressed | pointerup | Launch / Focus | 未运行 → 弹跳 600 ms；已运行 → 聚焦 |
| Pressed | 位移 > 4 px | DragSource | `scale 1.08 / rotate −1.5°`，阴影抬起 |
| DragSource | 悬停到槽位 | DragReorder | 邻居 `translateX ±pitch`，200 ms |
| DragSource | 离开 pill > 24 px | DragDetach | 不透明度 0.85，光标区提示「移除」 |
| DragReorder | pointerup | Settling | 320 ms spring 落定 |
| DragDetach | pointerup | Removing | 160 ms 淡出 + 缩到 0.86 |
| 任意 | contextmenu | MenuOpen | 放大冻结，菜单 250 ms |
| MenuOpen | Escape / 外部点击 | Idle | 150 ms 淡出 |

运行态是**叠加层**而不是状态：`running` 与 `hover`、`drag` 可同时存在；`disabled` 与其余状态互斥。

### 3.2 窗口级状态机（与 `autohide.rs` 对齐）

`Hidden`（2 px 边缘条）→ `Revealing`（200 ms 位移+淡入）→ `Visible` → `Concealing`（200 ms）→ `Hidden`。
`Suppressed`（全屏让位）强制 `Hidden`，解除后按开关回到 `Hidden` 或 `Visible`。
美术侧只需保证：收起条是 2 px、与任务栏不同色（用 accent 10%），否则用户找不到它。

### 3.3 放大传播

```
scale(i) = 1 + (peak − 1) × exp( −((d / radius) / falloff)⁴ )
d        = |center_i − pointerX|
radius   = magnifyRadiusIcons × (iconSize + iconGap)   // 默认 2.2 格
falloff  = 0.45
```

峰值 1.6 时：0 格 1.600 · 0.5 格 1.562 · 1.0 格 1.212 · 1.5 格 1.003 · 2.2 格 1.000。
`transform-origin: bottom center`（底部停靠）；左右边缘改为指向屏幕内侧。

这条曲线很窄：只有正邻接会明显抬起。想让联动更宽，改 `magnifyRadiusIcons` 到 3.0，不要改 falloff（会改变峰形）。

### 3.4 时长表

| 动作 | 属性 | 时长 | 曲线 |
|---|---|---|---|
| 放大跟随 | transform | 160 ms | `cubic-bezier(.22,1,.36,1)` |
| 离开复位 | transform | 320 ms | `cubic-bezier(.34,1.36,.64,1)` |
| 按下 / 松开 | inner scale | 90 / 120 ms | signature / spring |
| 气泡入 / 出 | opacity + Y | 150 / 50 ms | signature / linear |
| 菜单入 / 出 | opacity + scale | 250 / 150 ms | signature |
| 自动隐藏 | translateY | 200 ms | ease-out / ease-in-out |
| 角标 | scale .4 → 1 | 500 ms | spring |
| 启动弹跳 | Y 0 → −34 → 0 ×2 | 600 ms | spring |
| 拖拽让位 | translateX | 200 ms | signature |
| 拖出移除 | opacity + scale | 160 ms | ease-in |
| 面板切换 | opacity + Y 8 px | 220 / 160 ms | signature |

### 3.5 性能纪律

- 只动 `transform` / `opacity`。不动 `width` / `height` / `top` / `left`。
- `will-change: transform` 只加给**正在放大**的磁贴，离开即撤。
- 颗粒只挂在 `::after` 且 `pointer-events: none`，不挂滚动容器。
- 倒影是同一图层的缩放副本，不额外发起解码；但 10 个磁贴 + 倒影 = 20 层，放大时保持合成器路径。
- 降级守卫：`prefers-reduced-motion` 时全部归零或改纯 opacity，**优先于任何单个效果**。

---

## 4. 设置面板

### 4.1 信息架构

按需求分五栏。现有实现是 Appearance / Behaviour / Apps / Advanced 四栏，映射如下：

| 新栏 | 收纳 | 来自旧栏 |
|---|---|---|
| General | 语言、开机自启、应用管理（10 槽位）、配置文件位置 | Apps + Advanced |
| Appearance | 材质、主题、强调色、尺寸、圆角、玻璃、倒影、图标形状 | Appearance |
| Behaviour | 放大、动效档、提示、运行点、角标、自动隐藏、边缘 | Behaviour |
| Extensions | 插件列表、启用/排序、权限、崩溃状态 | 新增 |
| About | 版本、更新通道、崩溃恢复、第三方声明、重置 | Advanced |

> 如果 App 管理在 General 里显得拥挤，把它拆回独立第 6 栏比压缩行高更好，10 个槽位需要横向空间。

### 4.2 字段表

**Appearance**

| 字段 | 控件 | 范围 / 默认 | 美术影响 |
|---|---|---|---|
| `style` | 分段（Glass / Bare / Frost） | glass | 决定底板四层、倒影与间距 |
| `theme` | 分段（Dark / Light / System） | dark | 整包 token 切换 |
| `accent` | 色块 ×6 + 自定义 | `#7c9be0` | 焦点环、选中态、环仪表 |
| `iconSize` | 滑杆 | 32-80 / 48 | 窗口宽度与高度 |
| `iconGap` | 滑杆 | 0-24 / 8 | 同上 |
| `paddingH` / `paddingV` | 滑杆 | 4-32 / 2-24 | 同上 |
| `pillRadius` | 滑杆 | 0-64 / 34 | 默认跟随高度 |
| `iconShape` | 分段（Squircle / Rounded） | squircle | `clip-path` 或 `border-radius` |
| `iconRadius` | 滑杆 | 0-50 / 22 | 仅 Rounded 生效 |
| `fillOpacity` | 滑杆 | 0.15-1 / 0.62 | 玻璃填充 alpha |
| `tint` | 色块 | `#161a26` | 玻璃底色 |
| `reflection` | 复选 | 开 | 是否绘制倒影带 |
| `reflectionStrength` | 滑杆 | 0-1 / 0.25 | 倒影不透明度 |
| `separatorStyle` | 分段（Hairline / Dot） | hairline | 分隔线形态 |

**Behaviour**

| 字段 | 控件 | 范围 / 默认 | 美术影响 |
|---|---|---|---|
| `magnify` | 复选 | 开 | 关闭后不写 transform |
| `magnifyPeak` | 滑杆 | 1.0-2.0 / 1.6 | 峰高 |
| `magnifyRadiusIcons` | 滑杆 | 1-5 / 2.2 | 作用半径 |
| `profile` | 下拉（Elastic / Restrained） | vitro | 换曲线档 |
| `reducedMotion` | 复选 | 关 | 强制关闭全部动效 |
| `tooltips` / `tooltipDelayMs` | 复选 / 滑杆 | 开 / 500 | 气泡 |
| `showRunningDots` / `showBadges` | 复选 | 开 | 运行点 / 角标 |
| `autohide` / `showDelay` / `hideDelay` | 复选 / 滑杆 | 开 / 300 / 500 | 收起条 |
| `yieldFullscreen` | 复选 | 开 | 全屏让位 |
| `edge` | 分段（Bottom / Left / Right） | bottom | **行为未实现前置灰** |

**General / Extensions / About**：字段较少，沿用现有表单样式；插件页建议用两列（已安装 / 可安装），与 Apps 页同构。

### 4.3 即时预览契约

- 预览与真实 Dock 读同一组 CSS 变量；预览不是另一套样式。
- 拖动滑杆立即生效，停止 300 ms 后落盘（现有行为）。
- 材质、主题、强调色是**枚举切换**，不参与防抖，点击即写入。
- 尺寸类改动会改变窗口几何，因此滑动过程中只更新预览，松手后提交（避免每帧 resize 透明窗口）。
- 错误就近显示在字段下方；保存失败保留用户输入，不清空。

### 4.4 控件美术规格

| 控件 | 规格 |
|---|---|
| 分段控件 | 容器 `rgba(255,255,255,.05)`，选中 `13%` + 1 px 内高光 |
| 滑杆轨道 | 4 px，圆角 2，轨道白 10%，填充 accent |
| 滑杆拇指 | 14 px 圆，白 92% + 1 px 内高光 + 1 px 投影 |
| 色块 | 34 × 24，圆角 7，1 px 内高光 + 1 px 外描边 12% |
| 复选 | 15 px，圆角 4，选中 accent + 深色勾 |
| 分组标题 | 10 px 等宽，字距 0.08em，全大写，`--muted` |
| 行高 | 字段行 28 px，分组间距 20 px，全部 4 的倍数 |

---

## 5. 技术选型

### 5.1 三栈对比（只看影响美术的部分）

| | WPF | WinUI 3 | Tauri + Web |
|---|---|---|---|
| 矢量资产 | XAML Path，需从 SVG 转 | Win2D / Composition，需转 | 原生 SVG + CSS |
| 玻璃 | DWM interop，手写 | Mica/Acrylic 内置 | 绘制；原生模糊需副窗 |
| 形状遮罩 | Clip + Geometry | Composition clip | `clip-path` |
| 动效 | Storyboard，60 fps 需谨慎 | Composition animation，强但啰嗦 | CSS transform，合成器 |
| 迭代速度 | 低 | 中 | 高 |
| 内存实测 | 低 | 中 | 现有门禁已过（5.1 MB host / 179 MB 全树 / 0.04% 空闲） |

**推荐保持 Tauri**，理由不是"已经写了"，而是三条：

1. 这套美术系统是 CSS 变量 + SVG。换栈意味着 1200 行资产全部重写，而视觉收益为零。
2. 唯一 Tauri 做不到的是**桌面模糊**。但 WinUI 3 的 Mica/Acrylic 也只作用于窗口矩形，同样做不出 pill 形状，换栈并不能解决 T18，只是把问题挪个位置。
3. 现有的内存与 CPU 门禁已经达标，且输入层（hit-test / pointer_watch）是纯 Win32，与渲染栈无关。

### 5.2 不注入 Explorer

- 注入（DLL / 子类化 taskbar）在 Windows 功能更新后极易失效，且会被杀软标记。
- 产品不要求替换任务栏；独立置顶窗口 + `skipTaskbar` 已经满足「Dock 在桌面上」。
- 不注入的代价是无法与任务栏合并；这本来就是产品定位的选择，不是缺陷。

### 5.3 资产格式

SVG 源 + CSS 变量，理由：

- 一份源，任意 DPI，不需要 1x/2x/3x 导出。
- 变量层让主题切换是换 token，不是换皮肤。
- 全部纯文本，可 diff、可 review、可单独交给设计师。

### 5.4 两处刻意的偏离

| 常见建议 | 本包选择 | 理由 |
|---|---|---|
| 用 Geist / Satoshi 等 Web 字体提升质感 | `Segoe UI Variable` + `Cascadia Mono` | Windows 原生光学尺寸字体，零加载、ClearType 对齐最好；179 MB 的运行时不该为一个 13 px 标签多背一个字体文件 |
| 用 `@phosphor-icons/react` 或 `@radix-ui/react-icons` | 自绘 SVG sprite | 不新增依赖（用户要求不改现有工程）；现有 `Glyph.tsx` 已是 24/1.7 规范，换包会引入第二套网格 |

---

## 6. 风险与取舍

### 6.1 亚克力：真正的阻塞点

`window-vibrancy::apply_acrylic` 会模糊**整个窗口矩形**，包括上方 34 px 的放大 headroom 和下方 40 px 的倒影带。结果是图标周围出现一圈被模糊的桌面，比不模糊更脏。

三条路，按代价排序：

1. **不做**（当前）。绘制玻璃在深色壁纸上已经足够，浅色壁纸下靠 tint 提亮。
2. **pill 形状的原生副窗**。一个独立的分层窗口，用 region 裁成 pill，主窗口继续透明。代价：多一个 HWND、两次 DPI 换算、副窗与主窗的 z 序与焦点要额外处理。
3. **只在关闭放大时启用**。headroom 消失后整窗 ≈ pill，模糊范围问题自然消失。代价是功能耦合，用户无法同时要「真模糊」和「放大」。

本包把方案 1 作为默认风格 A，方案 2 作为可选风格 C（裁不出药丸时自动降级为 A）。方案 3 不建议。

### 6.2 `backdrop-filter` 不能模糊桌面

这是最容易踩的坑：在 Tauri 的 WebView 里，`backdrop-filter` 只能模糊 **DOM 中位于其后的内容**，模糊不到桌面壁纸。任何"先用 CSS 试试"的方案都会得到一张半透明纯色。

资源包的 acrylic 样张在预览页里看起来是真的，因为预览页的壁纸是 DOM 元素。产品里必须走原生。

### 6.3 浅色主题的对比度

玻璃背后的壁纸不可知。规则：

```
L < 0.35  → 白字 100%
0.35-0.6  → 白字 + 1 px 柔影（black 40%，offset 1）
L > 0.6   → 深字 #20242e
```

运行点在浅色下必须加 1 px 浅色描边（`--running-dot-ring`），否则 4 px 的白点在白壁纸上等于不存在。分隔线同理，浅色下用 `rgba(18,23,36,.14)`。

### 6.4 图标授权

HANDOVER §7.3 已记录：旧 `vitro` 图标没有可验证授权，不能进发布包。本包的 37 个字形是为本产品绘制的线性几何，可随产品发布；**不要**把第三方图标包（Phosphor / Radix / 各家 dock 项目的素材）复制进来。

### 6.5 倒影与放大的叠加成本

每个磁贴在放大时同时持有：缩放图层 + 倒影图层。10 个磁贴 = 最多 20 层。当前门禁能过，但这条线不要越：不要给倒影加 blur，不要给每个磁贴加独立滤镜。

### 6.6 拖出移除的歧义

「拖出 = 删除」是一个不可逆操作。视觉上必须给出三级提示：

1. 离开 pill 24 px：磁贴抬升 + 不透明度 0.85（还没删）。
2. 离开 80 px：pill 顶部出现「移除」提示条，磁贴缩到 0.9。
3. 松手：160 ms 淡出，并在通知区给 5 秒撤销。

没有第 3 步就不要做这个交互。

### 6.7 左右边缘

美术规格已给（`sheet/index.html#edges`），但行为未实现（多显示器、侧边几何、放大方向都要改）。**设置项在行为就绪前必须置灰或隐藏**，否则就是一个必然失败的承诺。

### 6.8 英文界面与文案长度

现有设置页是英文。若改中文，注意：Segoe UI 的中文回退是微软雅黑，字重与行高会变；字段标签列固定 124 px 可能截断。建议中文标签控制在 6 字以内，或把标签列改成上下结构。

---

## 7. 下一步实现清单

### P0 · 视觉对齐（零行为风险）

| # | 任务 | 验收 |
|---|---|---|
| 1 | 用 `--squircle-clip` 替换 `border-radius` | 32/48/80 px 下无可见折角；阴影仍在（阴影在父层） |
| 2 | 接入 `tokens/material.css` 色阶 | 全仓不再出现字面量 `#7fa4ff`、`#161a26`（除 token 文件） |
| 3 | 接入浅色 token 与运行点描边 | 白色壁纸下运行点对比 ≥ 3:1；设置页可读 |
| 4 | 用 `art/dock-glyphs.svg` 扩充 `Glyph.tsx` | 仍是 24 / 1.7；现有 6 个字形无回归 |

### P1 · 新能力

| # | 任务 | 验收 |
|---|---|---|
| 5 | 拆出语义动效 token | `prefers-reduced-motion` 覆盖率 100%（新增用例） |
| 6 | `material` 设置项 + 解释文案 | 未实现副窗前，Acrylic 选项置灰并说明原因 |
| 7 | `iconShape` 设置项 | 切换即时生效，预览与真实 Dock 一致 |
| 8 | 设置页改五栏 + Apps 面板 | 字段无丢失；保存/重置行为不变 |

### P2 · 扩展面

| # | 任务 | 验收 |
|---|---|---|
| 9 | 插件磁贴渲染器（ring / pill / badge / dots） | 互斥规则有单元测试；数值用等宽数字 |
| 10 | 插件崩溃态与恢复提示 | 崩溃不闪白；提示 5 秒可关闭 |
| 11 | 拖出移除 + 撤销 | 三级提示齐全；撤销 5 秒 |
| 12 | 从 `art/mark.svg` 导出 ico / png | 已完成：`icons/`，16 到 512 八档加 `dock.ico` |

### 总验收

| 维度 | 门槛 |
|---|---|
| 流畅度 | 放大期间 60 fps，掉帧 < 1%；无 layout thrash；`will-change` 只在放大中的磁贴上 |
| DPI | 100 / 125 / 150 / 175 / 200% 下 1 px 线宽视觉一致；图标按 `round(size × scale)` 取源 |
| 多屏 | 跨屏拖动后 pill 与图标不模糊；不同 DPI 的显示器间切换不重置位置 |
| 异常恢复 | sidecar 崩溃后 Dock 表面不闪；设置写失败有 inline 错误；崩溃恢复提示不阻塞交互 |
| 可定制闭环 | 每个设置项都能在 2 次点击内看到效果；每项都有默认值、范围、持久化与重置路径 |
| 性能预算 | host private < 30 MB；全树 < 200 MB；空闲 CPU < 1% |

---

## 8. 资产索引

```
design-art/
├─ README.md                    本文
├─ tokens/
│  ├─ material.css              色阶、玻璃四层、浅色、亚克力、阴影、纹理
│  ├─ motion.css                时长、曲线、语义动画、reduced-motion 守卫
│  └─ icon.css                  几何、squircle、运行点/角标/分隔线/环
├─ art/
│  ├─ dock-glyphs.svg           37 个字形的 sprite + 展示板
│  ├─ shapes.svg                squircle、环仪表、药丸、角标、倒影遮罩、投放槽
│  ├─ docklet-tiles.svg         6 个插件磁贴 × 深/浅两套
│  ├─ grain.svg                 颗粒纹理与降级写法
│  ├─ mark.svg                  512 px 产品标记
│  └─ mark-small.svg            16 到 32 px 简化标记
├─ icons/                       导出的 PNG 与 dock.ico
├─ tools/                       导出脚本与图标渲染页
├─ themes/                      三套可选风格
├─ 3d/                          three.js 三维 Dock 层与视觉稿
└─ vendor/three/                three.js r160 本地副本（MIT）
├─ sheet/
│  ├─ index.html                规范页（材质 / 色彩 / 形状 / 字形 / 解剖 / 状态 / 动效 / 设置）
│  ├─ sheet.css
│  └─ sheet.js                  放大曲线绘制 + 目录高亮
└─ preview/
   ├─ dock.html                 可交互 Dock 预览
   ├─ dock.css
   └─ dock.js                   真实放大公式、气泡、右键菜单、键盘导航
```

### 怎么打开

- 直接双击 `preview/dock.html` 或 `sheet/index.html`（Edge / Chrome 均可）。
- 页面通过相对路径引用 `../tokens/*.css` 与 `../art/*.svg`，**不要单独移动某个文件**，整体目录一起移动。
- 预览页的壁纸是 DOM 渐变，用于观察玻璃对比；它不是真实桌面的替代品，最终效果仍需在真实窗口里验收。

### 一个容易被忽略的细节

`--squircle-clip` 是 64 点百分比多边形，与 `art/shapes.svg` 里 48 段三次贝塞尔的路径数值等价。用哪一个取决于场景：CSS 用多边形（任意尺寸），导出位图用路径（曲线更顺）。两者都由超椭圆 n=5 生成，改一个必须同步改另一个。
