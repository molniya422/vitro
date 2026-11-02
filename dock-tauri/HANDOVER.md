# Dock Tauri 接手文档

> 更新：2026-11-01
> 分支：`codex/vitro-appruntime`（HEAD `067d3ac`）
> 项目根：`D:\dockyh\dock-tauri`
> 规格：`D:\dockyh\docs\superpowers\specs\2026-10-26-tauri-python-dock-design.md`
> Phase 3 计划：`D:\dockyh\docs\superpowers\plans\2026-10-26-dock-tauri-phase3.md`
> Phase 4 计划：`D:\dockyh\docs\superpowers\plans\2026-10-28-dock-tauri-phase4-drag.md`
> Phase 5 计划：`D:\dockyh\docs\superpowers\plans\2026-10-29-dock-tauri-phase5-shelf.md`

## 0. 一句话状态

Phase 3（可配置外观 + 原生输入修复 + 完整设置页 + 应用管理）、Phase 4（dock 拖动重排 +
从 Launchpad 拖入）和 Phase 5（Shelf 文件暂存 + 美术图标接入）已在工作区完成并验证：
Rust 132、Vitest 80、Playwright 20、pytest 9 全部通过；release 实测 **5.9 MB host /
186.6 MB 全树 / 0.03% 空闲 CPU**（先展开一次再收起，sidecar 已按 45 s 空闲规则退出）；
MSI 已重新构建但**未安装**。

**注意：本轮所有代码改动都还没有提交。** HEAD 仍是 `067d3ac`，工作区同时包含更早的
Phase 1/2 未提交工作和一些与本产品无关的既有改动（`vitro/` 等）。提交前必须逐项审查，
不要 `git add -A`。

## 1. 快速开始

### 1.1 环境

| 项 | 路径 / 要求 |
|---|---|
| Windows | Windows 11，WebView2（系统自带） |
| Rust | 1.89+，`D:\rust` |
| Node | 20+，npm |
| Python | `D:\jiqixuexi\anaconda\python.exe`（Anaconda），带 `psutil`；打包时还要 `pyinstaller` |
| 屏幕（开发机） | 2560×1440，150% 缩放 |

每次开新 shell 需要：

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"      # Tauri CLI 会调用 cargo
cd /d/dockyh/dock-tauri
```

### 1.2 开发运行

```bash
npm install
npm run tauri dev
```

- 自动隐藏默认开启，启动时 dock 是屏幕底部一条 2px 的条；鼠标移到屏幕底边才展开。
- 单一窗口承载 `dock` / `launchpad` / `monitor` / `settings` 四个路由。
- 开发模式下不要用裸 `cargo run` 代替；前端由 Vite 提供。

### 1.3 测试

```bash
# Rust（132）
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml

# 前端（78）
npm test

# Playwright（20，启动 Vite，使用假的 Tauri IPC）
npm run test:e2e

# Python sidecar（9）
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

E2E 不驱动真实 Tauri 窗口，只验证 React/路由/IPC 契约；原生点击穿透、自动隐藏距离、
全屏让位靠单元测试 + 手工验收（见 §8）。

### 1.4 构建

```bash
# 只出前端
npm run build

# release exe + MSI（会先跑 npm run build）
npm run tauri build
```

**不要**直接 `cargo build --release`：没有 Tauri CLI 注入的 `custom-protocol` feature，
exe 不会内嵌前端，会显示 Chromium 错误页。

产物：

- `src-tauri/target/release/dock-tauri.exe`（约 3.3 MB）
- `src-tauri/target/release/bundle/msi/Dock Tauri_0.1.0_x64_en-US.msi`（约 9.5 MiB）
- `src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe`（PyInstaller sidecar，约 7.8 MB）

`target/`、`dist/`、`binaries/monitor-*.exe` 都在 `.gitignore` 里，不要提交。

## 2. 当前状态

### 2.1 已完成（Phase 3）

- 设置模型 v2：`icon_size`、间距、内边距、pill 圆角、tint、透明度、图标圆角、倒影强度、
  放大峰值/范围、提示延迟、运行点/角标、自动隐藏延迟、全屏让位、入口显隐等。
- 设置页四栏：Appearance / Behaviour / Apps / Advanced，实时预览，300 ms 防抖自动保存；
  `Save` 与 `Close` 会立即落盘；初始加载和 StrictMode 重挂载不会误写。
- 应用管理：设置页添加/删除/上下排序；dock 右键菜单删除（系统磁贴不可删）。
- 原生命中测试：透明 headroom/reflection 点击穿透；pill 可交互；右键菜单区域也算交互区。
- 自动隐藏：按 pill 判定，而不是整个窗口；离开 pill 即开始计时；从面板返回时重新计时。
- 全屏让位：真正全屏的前台窗口会隐藏 dock 窗口，退出后恢复边缘条（最大化窗口不算全屏）。
- 图标：按 `round(icon_size × DPI)` 提取（150% 下 48 CSS px 用 72px 源图），懒加载快照，
  1 Hz 同步不再读写图标。
- 启动性能：Launchpad 目录懒加载；sidecar 启动 500 ms 后预热。
- 视觉：图标阴影/按压反馈、玻璃渐变与内阴影、提示气泡、倒影/运行点/角标可配置。
- 文档：README 与设计规范已更新到实际状态；Phase 3 计划含实现状态说明。

### 2.1.1 Phase 4 新增（2026-10-28）

- dock 拖动重排：按住图标移动超过 4 CSS px 进入拖动；被拖图标跟随指针，源与目标之间的
  图标让出一个槽位；松手提交 `reorder_pins`，并用本地 pending 顺序避免落盘期间回跳。
- 拖动期间 `set_drag_active(true/false)` 锁定自动隐藏，并让原生 watcher 保持整个窗口可交互
  （否则指针移到 pill 上方时窗口会变点击穿透，webview 收不到 move，拖动会卡住）。
- 拖动松手不会触发启动：`suppressClick` 吞掉随后的 click；窗口失焦会取消拖动并解锁。
- Launchpad 拖入：拖动 tile 时在屏幕底部出现 dock 形状的投放条，按指针位置预览插入槽；
  在底部 140 px 内松手 → `pin_app` 后 `reorder_pins` 插到该槽；已固定的应用直接移动；
  在其它位置松手取消。单击 tile 仍是启动；`+` 按钮仍是可访问的添加入口。
- 新纯函数 `src/dock/drag.ts`（`reorderIds` / `targetIndex` / `shiftFor` / `placeAt`）+
  14 个 Vitest、2 个 Playwright 用例。

### 2.1.2 Phase 5 新增（2026-10-29）

- **Shelf 文件暂存**：监听 Tauri 的 `tauri://drag-drop`（`getCurrentWebview().onDragDropEvent`），
  把拖到窗口上的文件/文件夹加入本次会话的暂存列表，自动切到 `shelf` 路由。
- 新 `src-tauri/src/shelf.rs` 纯路径逻辑：大小写不敏感去重、上限 12、显示名；对应 6 个 Rust 测试。
- 新命令：`get_shelf` / `shelf_add` / `shelf_remove` / `shelf_clear` / `open_shelf_item`。
  打开文件只接受已经在 shelf 里的路径（防止 webview 变成通用启动器）；图标按 64 CSS px 提取，
  复用 Shell 图标磁盘缓存。
- 新面板 `src/shelf/Shelf.tsx`：空状态、列表、点击打开、逐项删除、Clear；面板本身也接受文件拖入。
- dock 新增 shelf 入口（tray 字形）；`show_shelf_entry` 进入设置 schema 3（默认 true），
  `CURRENT_SCHEMA` 2 → 3，v1/v2 文件按默认值升级，并在下次保存时写回 3。
- 美术接入：应用图标（exe/安装包/窗口图标）换成 `design-art/icons/dock.ico` + 对应 PNG；
  shelf 字形采用 `design-art/art/dock-glyphs.svg#g-tray`（24 网格 / 1.7 描边）；
  gear / chart / sliders 三个现有字形也按同一导出对齐（folder / grid 原本一致）。
- 启动脉冲：点击应用磁贴时图标做一次 700 ms 弹跳；拖动松手不会触发；`reduced_motion`
  或系统 reduced-motion 偏好都会关闭动画。
- 运行中应用点击聚焦：`launch_item` 先解析 AppEntry、枚举窗口，按“精确进程路径 → 文件名”
  匹配；优先可见窗口，最小化的先 `ShowWindow(SW_RESTORE)`，再 `SetForegroundWindow`；
  没有任何窗口时才退回 ShellExecute。这样点击运行中的应用不会再开第二个实例。
  **窗口快照必须排除 `Progman` / `WorkerW` / `Shell_TrayWnd`**：桌面与任务栏也属于
  explorer.exe 且是可见窗口，不排除时“优先可见窗口”会选中桌面，最小化的资源管理器永远
  不会被还原（实机测试踩过）。
- 测试：Rust 122 → 132，Vitest 72 → 80，Playwright 18 → 20。

### 2.2 尚未完成 / 开放项

- **真实桌面模糊（Acrylic）**：T18 仍未决策。`window-vibrancy::apply_acrylic` 会模糊整个
  窗口矩形（包括透明 headroom 和反射带），不能只覆盖 pill。当前只是可调 tint + 透明度的
  绘制玻璃。三个候选方案见 Phase 2 计划 T18。
- T23 剩余精修：真正的 squircle 遮罩（当前是可调 `border-radius`）。启动脉冲已在 Phase 5 补上。
- 托盘镜像、Docklet、MSIX/AUMID 身份、自定义图标文件。
- Shelf 目前是会话级、不能拖出；美术包的浅色主题、material/iconShape 设置、3D 样张未接入。
- 多显示器、左/右/上边缘、开机自启、退出入口（当前没有托盘/退出菜单，只能结束进程）。
- 复核/提交工作区改动（见 §11）。

### 2.3 当前机器状态

- 没有残留的 `dock-tauri` 或 Vite 进程；1420 端口空闲。
- 设置文件 `%APPDATA%\DockApp\settings.json` 当前是**用户自己的状态**：schema 3、10 个固定项
  （顺序已由用户调整）、`show_shelf_entry: true`、`autohide: true`、默认外观。
  **不要用旧备份覆盖它。**
- release exe 已停止运行；没有残留的 `monitor` sidecar。
- MSI 已重新构建（含 Phase 4 拖动、Phase 5 Shelf、sidecar 空闲回收和新应用图标），
  **没有安装**。安装会修改用户机器，属于用户决定。

## 3. 架构

### 3.1 进程与窗口

- 一个 Tauri 窗口（label `dock`）：透明、无边框、always-on-top、skipTaskbar。
- React 负责全部像素，内部切换四个路由，所以只有一个 WebView2 renderer。
- Python 是 stdio JSON-RPC sidecar（`{"cmd":"sample"}` → 一行 JSON），不是 HTTP 服务。
- Rust 管窗口几何、自动隐藏、命中测试、图标、应用目录、设置和 sidecar 监督。

### 3.2 配置数据流

```
settings.json (snake_case, schema 3)
        │  load + sanitize
        ▼
   Settings ──► DockMetrics ──► window geometry (physical px)
        │
        └────► DockAppearance ──► React CSS variables + layout
```

- Rust 是唯一几何来源；React 不自己算窗口尺寸。
- IPC 使用 camelCase 的 `SettingsView`；文件仍是 snake_case。
- `pinned` 和 `schema_version` 由 Rust 持有，设置表单不能覆盖。
- 高度 = `icon_size + 2 × padding_v`；宽度由图标数、间距、内边距、可见入口数推导。
- 设置窗口固定 820×680 逻辑像素（按 DPI 缩放）；Launchpad 填工作区；monitor 是 dock 高度
  + 320 逻辑像素。

### 3.3 输入 / 点击穿透（关键）

窗口比 pill 大（上方 magnify headroom，下方 reflection），所以不能用 CSS 决定命中：

1. React 测量 pill 的**宽度**（CSS px），通过 `set_hit_rects(pillWidth, overlayRects, scale)`
   上报；右键菜单作为额外的 overlay rect。
2. Rust 用窗口矩形 + `DockMetrics` 推导 pill 的物理位置和高度。位置/高度不信任 CSS x/y，
   因为 auto-hide 会在两种 viewport 布局之间切换；宽度是稳定的。
3. `pointer_watch.rs` 每 16 ms（靠近 dock）或 100 ms（远离）读 `GetCursorPos`，切换
   `set_ignore_cursor_events`：
   - 可点击区 = pill + 打开的菜单；
   - 指针存在区 = pill 向下延伸到窗口底部（反射带）；反射带保持点击穿透，但不会让 dock
     在刚展开时立刻收起。
4. 右键菜单打开时会把菜单 rect 一并上报，否则菜单区域会被穿透。
5. 拖动图标期间 `set_drag_active(true)`：watcher 强制 `ignore=false` 并跳过自动隐藏/全屏
   判定，直到 `set_drag_active(false)`。没有这一步，指针拖到 pill 上方 headroom 时窗口会
   变成点击穿透，webview 收不到 pointermove，拖动会冻结。窗口失焦会取消拖动并清除标记。
6. 全屏检测每 500 ms：取前台窗口矩形和主屏矩形比较；排除桌面 `Progman`/`WorkerW`。
   全屏时隐藏整个窗口并禁用命中测试。

调试输入相关问题时，先看 `pointer_watch.rs`，再看 `hit_test.rs` 的纯函数测试。

### 3.4 自动隐藏状态机

`autohide.rs` 是纯状态机，时间由调用方传入：

- `new_with_delays(enabled, show, hide)`：开启时初始 Hidden。
- `PointerEntered` / `PointerLeft` 只在 pill（或 hot strip）边界变化时触发。
- `Tick` 到 deadline 才执行 Reveal/Collapse。
- `SetSuppressed(true)` 强制 Hidden（全屏让位），取消未决定时；解除后按 auto-hide 开关
  回到 Hidden 或 Visible。
- 面板路由期间不参与自动隐藏；返回 dock 时 `last_inside=true`，若指针不在 dock 上，下一帧
  会发 PointerLeft 并正常计时收起（这是实测发现并修掉的“关闭设置后一直不隐藏”问题）。

### 3.5 图标

- 设备像素：`round(icon_size × monitor.scale)`，限制在 16–256（Shell jumbo 上限）。
- `icon_cache.rs` 的 `IconSnapshot` 与 catalog 并行、按 index 懒加载：dock 只加载 pinned，
  Launchpad 首次打开才加载全部；失败也标记 loaded，避免每秒重试。
- 磁盘缓存：`%APPDATA%\DockApp\icons`，key = sha256(小写路径 + "@" + size)，内容是 PNG，
  读出来转 data URL。
- 如果更换了应用图标但显示旧图，删掉上述缓存目录即可；改变 icon size 会产生新 key。
- `SHELL` 图标路径里的 `/` 会先转成 `\`，`SHGetFileInfoW` 不接受正斜杠路径。

### 3.6 Python sidecar

- 冻结版：`src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe`，Tauri 打包后放在 exe 旁边并
  命名为 `monitor.exe`。
- 开发回退：`commands::sidecar_spec()` 找不到 `monitor.exe` 时用 Anaconda Python +
  `backend/run_sidecar.py`；可用 `DOCK_PYTHON` 覆盖解释器路径。
- `Supervisor` 负责重启、冷却和类型化错误；崩溃后下一次采样会重拉。
- 启动 500 ms 后预热一次采样；**45 s 没有采样就杀掉子进程**（`reap_if_idle`，1 Hz 线程
  调用），`set_route("monitor")` 时 `warm()` 重新拉起，所以关掉 monitor 后不常驻 Python。
  首次打开 monitor 若刚好在回收后，会有约 1 s 的 PyInstaller 启动延迟。
- **内存测量**：空闲时 sidecar 已被回收，全树约 187 MB；回收前同状态约 203 MB。

## 4. 关键文件地图

```
dock-tauri/
  src-tauri/src/
    appearance.rs      设置 → DockMetrics / DockAppearance（纯函数）
    geometry.rs        窗口物理矩形（纯函数）
    window_geom.rs     各路由尺寸（纯函数）
    autohide.rs        自动隐藏状态机（延迟、抑制）
    hit_test.rs        Rect、pill_rect、click-through 数学（纯函数）
    foreground.rs      全屏判定（纯函数）
    settings.rs        schema 3、迁移、sanitize、原子写、SettingsView
    shelf.rs           Shelf 路径列表：去重、上限、显示名（纯函数）
    platform.rs        Win32 工作区/DPI/光标/前台窗口/窗口矩形
    apps.rs            Start Menu 扫描、.lnk 解析、启动、运行态匹配
    icon_cache.rs      懒加载 IconSnapshot + PNG 磁盘缓存 + data URL
    windows_tracker.rs EnumWindows 快照
    metrics.rs         sidecar Supervisor
    pointer_watch.rs   原生光标监视：穿透、自动隐藏、全屏让位、拖动锁定
    commands.rs        Runtime 缓存 + Tauri invoke 表面（含 set_drag_active）
    lib.rs             启动、1 Hz 同步、线程、命令注册
  src/
    types.ts           DockAppearance / SettingsView / DockState
    api.ts             invoke 包装
    App.tsx            路由、dock state 订阅、懒加载 catalog、Launchpad 落位
    dock/Dock.tsx      pill、放大、渲染、点击/右键、拖动重排、命中宽度上报
    dock/DockItem.tsx  单个图标、倒影/运行点/角标/提示、拖动抬起
    dock/ContextMenu.tsx
    dock/drag.ts       纯拖动数学：reorderIds / targetIndex / shiftFor / placeAt
    dock/layout.ts     cssVars / iconCenters / hexToRgba
    dock/magnify.ts    放大曲线
    dock/dock.css
    launchpad/Launchpad.tsx  搜索、拖入 dock 的投放条与 ghost
    launchpad/AppTile.tsx
    launchpad/launchpad.css
    shelf/Shelf.tsx     文件暂存面板（打开/删除/Clear、空状态）
    shelf/shelf.css
    settings/SettingsPanel.tsx  四栏设置页、自动保存、应用管理
    settings/validate.ts        范围校验 + 默认值（与 Rust 镜像）
    settings/preview.tsx        实时预览
    settings/settings.css
  e2e/                 Playwright + 假 IPC
  backend/             Python sidecar + pytest
  scripts/measure_memory.ps1
  README.md
  HANDOVER.md          本文
```

## 5. 设置 schema（v2）

文件：`%APPDATA%\DockApp\settings.json`（snake_case）。IPC 是 camelCase `SettingsView`。

| 文件键 | 范围 | 默认 | 说明 |
|---|---|---:|---|
| `icon_size` | 32–80 | 48 | 逻辑 px |
| `icon_gap` | 0–24 | 8 | 逻辑 px |
| `padding_h` | 4–32 | 16 | 逻辑 px |
| `padding_v` | 2–24 | 10 | 逻辑 px；bar 高度 = icon_size + 2× |
| `pill_radius` | 0–64 | 34 | px |
| `tint` | `#rrggbb` | `#161a26` | 玻璃底色 |
| `fill_opacity` | 0.15–1.0 | 0.62 | 玻璃填充 alpha（不是窗口 alpha） |
| `icon_radius` | 0–50 | 22 | % |
| `reflection` | bool | true | |
| `reflection_strength` | 0–1 | 0.25 | |
| `magnify` | bool | true | |
| `magnify_peak` | 1.0–2.0 | 1.6 | |
| `magnify_radius_icons` | 1.0–5.0 | 2.2 | 作用范围（图标数） |
| `profile` | `macos` / `vitro` | `vitro` | 动效档位 |
| `reduced_motion` | bool | false | |
| `tooltips` / `tooltip_delay_ms` | bool / 0–2000 | true / 500 | |
| `show_running_dots` / `show_badges` | bool | true | |
| `autohide` | bool | **true（新装默认）** | schema 1 文件保留原值 |
| `show_delay_ms` / `hide_delay_ms` | 0–1000 / 100–3000 | 300 / 500 | |
| `yield_fullscreen` | bool | true | |
| `show_launchpad_entry` / `show_monitor_entry` / `show_shelf_entry` | bool | true | 影响 leading tiles 和窗口宽度；`show_shelf_entry` 是 schema 3 字段，旧文件默认 true |
| `pinned` | `Vec<String>` | 首启 seed | 由 pin/unpin/reorder 命令维护 |

写入顺序：临时文件 → flush → sync → `.bak` 轮换 → rename。主文件损坏会回退 `.bak`；
两者都损坏会报类型化错误。手工编辑文件后，下次 `save_settings` 或重启才生效（运行中
的后端有内存缓存，不会轮询文件）。Rust 在 load/save 前都会 sanitize，越界值会被夹回范围。

## 6. 测试与性能

### 6.1 当前结果（2026-10-26，开发机）

| 层 | 数量 | 说明 |
|---|---:|---|
| Rust | 132 | 几何、外观派生、settings 迁移/sanitize、自动隐藏、命中矩形、全屏判定、图标、sidecar、pin 顺序、Shelf 路径逻辑、运行窗口聚焦匹配 |
| Vitest | 80 | 放大曲线、布局/CSS vars、设置校验、设置页、应用管理、Dock/ContextMenu、倒影/运行点开关、拖动数学与两种拖动手势、Shelf 面板、启动脉冲 |
| Playwright | 20 | 假 Tauri IPC：渲染、放大、提示、状态推送、设置页四栏、增删应用、右键菜单、dock 拖动重排、Launchpad 拖入、Shelf（含模拟文件拖入）、monitor |
| pytest | 9 | sidecar 协议、采样、启动器 |

性能（release，60 s settle，3 次中位数，自动隐藏开启，**先展开一次 dock 再移开指针**）：

| 指标 | 门禁 | 实测 |
|---|---:|---:|
| host private bytes | < 30 MB | 5.9 MB |
| 全树（WebView2 + Python sidecar + conhost） | < 200 MB | 186.6 MB |
| 空闲 CPU | < 1% | 0.03% |

测量口径说明：WebView2 只有在 pill 真正显示过一次之后才会栅格化完整 dock；只测“从未
展开”的收起态会低估占用（先前 179.2 MB 即此类）。当前口径先展开再收起，且 sidecar 已按
45 s 空闲规则退出（同一状态下未回收 sidecar 时约 203 MB）。

复测：先启动 release exe，再运行
`powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3`。
测量期间不要打开 monitor/Launchpad，否则会提高数值。

### 6.2 为什么数字比 Phase 1 好

- 1 Hz 同步只在运行状态/固定项/设置变化时构建 state，不再每秒遍历图标文件。
- `IconSnapshot` 内存复用；目录懒加载。
- 输入 watcher 远处 100 ms 轮询，近处才 16 ms。
- 只有一个 WebView2 renderer。
- sidecar 预热后常驻，但只在 monitor 打开时采样。

## 7. 发布

### 7.1 先冻结 sidecar

```bash
D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller   # 如未安装
powershell -ExecutionPolicy Bypass -File backend/build_sidecar.ps1
```

产物：`src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe`。
Tauri 要求 `externalBin` 文件名带 target triple；脚本已处理。

### 7.2 构建

```bash
npm run tauri build
```

- 只认这个命令；裸 `cargo build --release` 会得到错误页。
- 构建完成后检查 MSI 和 exe；**不要自动安装 MSI**。
- 当前 MSI：`src-tauri/target/release/bundle/msi/Dock Tauri_0.1.0_x64_en-US.msi`（9.48 MiB），
  未安装。

### 7.3 发布边界

- 不打包 `docs/prototypes/`、`vitro` 运行时源码、GlassDesk/BitDock/Nexus 等复制代码。
- 不打包 `target/`、`dist/`、sidecar 二进制到 Git。
- 应用图标现在来自 `dock-tauri/design-art/icons`（已复制到 `src-tauri/icons`），不要再使用
  `vitro` 的旧图标；美术包其余资产按需接入，不要整包打入安装包。

## 8. 手工 / 原生验收

E2E 覆盖不到真实 Win32 行为，至少每次改动输入层后做下面几项。

开发机坐标（2560×1440 @150%）：

- dock 窗口（4 个 leading tile）：`(674,1152)–(1886,1440)`，1212×288
- pill：顶部约 1278，底部约 1380；leading 中心 x ≈ 734（控制）/ 818（Launchpad）/
  902（monitor）/ 986（Shelf），第一个应用 ≈ 1070，pitch 84，y=1329
- 收起条：`(0,1438)–(2560,1440)`，2560×2
- 设置窗口：`(665,420)–(1895,1440)`，1230×1020
- Shelf / monitor 面板：`(674,672)–(1886,1440)`，1212×768

### 8.1 点击穿透

1. 用 Notepad 或其它窗口覆盖 dock 区域。
2. 点击 pill 上方 headroom（例如 1280,1200）和下方反射带（例如 900,1415）。
3. 目标窗口应保持前台；点击 pill 才应激活 dock。
4. 注意：合成鼠标事件（`SetCursorPos` + `mouse_event`）可能留下卡住的拖动状态；测试后发送
   一次 `LEFTUP`，必要时重启应用。

### 8.2 自动隐藏

1. 开启 autohide，确认启动时窗口为 2560×2。
2. 指针移到屏幕底边 → 300 ms 后应展开为 1128×288。
3. 指针移到 pill 上方但在旧窗口 headroom 内 → 500 ms 后应收起为 2560×2。
4. 关闭设置面板后指针不在 dock 上 → 应自动收起。

### 8.3 全屏让位

用一个无边框、`TopMost`、覆盖 `(0,0,2560,1440)` 的 WinForms 窗口作为前台窗口：

- 1 s 内 dock 窗口 `Visible=False`。
- 关闭该窗口后，dock 恢复为 2px 条（autohide 开）或完整 dock（autohide 关）。

### 8.4 设置页

1. 展开 dock，单击左侧控制磁贴 → 窗口应变为 1230×1020，四栏可切换。
2. 拖动 Icon size / opacity / tint / radius → 预览立即变化；停止 300 ms 后应显示 `Saved.`。
3. 关闭后 dock 应用新尺寸；重新打开设置值仍在。
4. Apps 栏添加/删除/上下排序；dock 右键菜单删除非系统项。

### 8.5 拖动重排与 Launchpad 拖入

1. 展开 dock（关闭 autohide 或把指针移到屏幕底边）。按住一个图标横移超过 4 px：
   图标抬起跟随指针，邻居让出槽位；松手后 `settings.json` 的 `pinned` 顺序改变，
   且**不会启动**该应用。
2. 拖动过程中把指针移到 pill 上方的 headroom：dock 必须保持可见并继续跟随。这是
   `set_drag_active` 的关键路径；若拖动冻结，先查这里。
3. 按住图标不移动直接松开：仍应正常启动（4 px 阈值）。
4. 拖到 pill 外竖直方向再松手：不应重排，也不应启动。
5. Launchpad 打开后拖一个 tile 到底部：出现 dock 形状的投放条，预览中该应用已插在
   指针所在槽；在底部 140 px 内松手 → 应用被插入 dock；在其上方松手 → 取消；
   单击 tile 仍是启动。
6. 合成鼠标事件可能产生不可靠的拖动（见 §9.12）。有条件时用真实鼠标复核。

### 8.6 Shelf

1. 展开 dock，单击 Shelf 入口（开发机约 986,1329）→ 窗口变为 1212×768，显示空状态和
   禁用的 Clear。
2. 从 Explorer 拖一个文件到面板上松手：列表出现该项，带真实 Shell 图标；点击应打开文件，
   × 删除，Clear 清空，Close 回到 dock。
3. 从 Explorer 拖文件到**可见的 pill** 上松手：应自动切到 Shelf 并显示该项。
4. 重启应用后 Shelf 应为空（会话级，不持久化）。
5. 注意 autohide 开启时窗口只有 2px 时可拖放区域几乎不可用：先把 dock 展开，或者拖到
   已打开的 Shelf 面板上。

### 8.7 截图 / 窗口探测

PowerShell 截图和 `GetWindowRect` 必须 DPI-aware，否则会看到 1707×960 的虚拟坐标：

```powershell
Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public class Dpi {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}'
[Dpi]::SetProcessDPIAware() | Out-Null
```

## 9. 已知限制 / 坑

1. **Acrylic 仍阻塞**（T18）。不要把 `apply_acrylic` 加到当前窗口上，它会把 headroom 和
   反射带一起模糊。要真实模糊，需要选择 T18 的方案二（pill 形状的原生玻璃副窗 + region）。
2. **Dock 上限 10 个应用**。拖动重排已实现；设置页的上下移动按钮保留为键盘/可访问路径。
   Launchpad 拖入的投放条是底部 140 px 的带，松手位置按“预览里最近的槽”计算。
3. **只有主显示器**，只支持底部；多显示器/其它边缘未实现。
4. **没有托盘/退出入口**。开发时用 Task Manager 或 `Stop-Process -Name dock-tauri` 结束；
   正式产品需要补一个退出方式。
5. **Store/MSIX 应用**没有经典 Start Menu target，运行态可能永远不亮；需要 AUMID。
6. **运行态匹配**先精确进程路径，再按文件名回退（Doubao 的快捷方式和实际 exe 不同目录）。
   同名二进制可能误匹配。
7. **窗口过滤**排除只有 `WS_EX_TOOLWINDOW` 窗口的应用。
8. **2px 收起条**覆盖任务栏最底部像素；全屏让位和可配置延迟可缓解，可关闭 autohide。
9. **Playwright 是 mock IPC**，不能证明 Win32 行为；原生部分靠纯函数测试 + 手工验收。
10. **设置页是英文**；如要中文界面，集中改 `SettingsPanel.tsx` 文案。
11. **开发时 Vite HMR 会重置 React 路由**；改完设置/路由代码后建议完整重启一次再截图验收。
12. **合成输入测试**：在路由切换/窗口缩放期间连续注入点击和移动，可能打开/关闭面板、
    拖动滑杆，造成看似“自己开关”的现象。2026-10-28 的实测中，`SetCursorPos` +
    `mouse_event` 在拖动验收时甚至自行打开了设置页、改动了滑杆并把某个固定项移出了 dock。
    因此：不要把合成输入当作拖动功能的判据；优先用 Playwright 验证逻辑，用真实鼠标做
    原生验收。实验前先备份 `settings.json`，实验后逐项检查并恢复用户当前值。
13. **当前工作区未提交**，且混有更早的未提交改动；见 §11。
14. **Shelf 是会话级**：最多 12 项，不持久化，没有拖出。文件只在窗口可交互的位置能接住拖放：
    可见的 pill 或已打开的面板；autohide 开启时要先展开 dock（2px 收起条几乎不可用作投放目标）。
15. **Shelf 图标按 64 CSS px 提取**，与 dock 的 icon_size 独立；修改图标缓存键或尺寸时注意
    两条路径共用 `%APPDATA%\DockApp\icons`。

## 10. 下一步建议

按优先级：

1. **决定 T18 Acrylic 方案**（保持绘制玻璃 / 原生玻璃副窗 / 仅关闭放大时启用）。
2. **提交与分组**：审查 diff，按 Phase 1–5 或功能分组提交；不要把 `vitro/`、`docs/` 里的
   无关旧改动、`ui-ux-pro-max-skill-main/` 等塞进同一提交。`design-art/` 是美术资源，
   按需只提交真正用到的文件（当前是 `icons/` 和 g-tray 的路径数据）。
3. **继续接美术包**：P0 的 squircle clip、token 色阶、浅色 token；P1 的 material/iconShape
   设置项（Acrylic 未实现前置灰）。推荐先做 squircle 与 token，再做主题切换。
4. T23 精修：启动脉冲、鼠标滚轮切换/滚动。
5. 托盘镜像、Docklet；Shelf 的拖出、持久化、文件夹展开。
6. MSIX/AUMID 身份，解决 Store 应用运行态和安装身份。
7. 中文界面 / 开机自启 / 退出入口。
8. 若提高 10 项上限，必须同时处理窄屏宽度 clamp 和溢出裁剪。
9. 每次重新打包后复测两个内存门禁和空闲 CPU；当前 MSI 已含 Phase 5。

## 11. 工作区与提交注意

当前状态：

- 分支 `codex/vitro-appruntime`，HEAD `067d3ac`。
- 当前约 50 个已修改文件 + 30 个未跟踪文件（其中包含与本产品无关或更早的改动；
  数量会随工作区变化，以 `git status` 为准）。
- 本轮的 Phase 3 新文件：
  - `src-tauri/src/{appearance,foreground,hit_test,pointer_watch}.rs`
  - `src-tauri/tests/unit/{appearance,foreground,hit_test,pins,settings_view}.rs`
  - `src/dock/{ContextMenu.tsx,DockItem.test.tsx,layout.ts,layout.test.ts}`
  - `src/settings/{preview.tsx,preview.test.tsx}`
  - `docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md`
- Phase 4 新文件：
  - `src/dock/{drag.ts,drag.test.ts}`
  - `docs/superpowers/plans/2026-10-28-dock-tauri-phase4-drag.md`
- Phase 3 修改的主要文件见计划 T19–T24 的 Files 列表；`README.md`、设计规范、
  `e2e/*`、`settings/*`、`dock/*`、Rust 的 settings/commands/lib/geometry 等都在其中。
- Phase 4 另改了：`src/dock/{Dock,DockItem}.tsx`、`src/dock/dock.css`、
  `src/launchpad/{Launchpad,AppTile}.tsx`、`src/launchpad/launchpad.css`、`src/App.tsx`、
  `src/api.ts`、`src-tauri/src/{commands,pointer_watch,lib}.rs`、`e2e/{dock,panels}.spec.ts`、
  `playwright.config.ts`、README、设计规范 §15。
- Phase 5 新文件：
  - `src-tauri/src/shelf.rs`、`src-tauri/tests/unit/shelf.rs`
  - `src/shelf/{Shelf.tsx,Shelf.test.tsx,shelf.css}`
  - `docs/superpowers/plans/2026-10-29-dock-tauri-phase5-shelf.md`
- Phase 5 另改了：`src-tauri/src/{settings,appearance,commands,lib,window_geom,metrics}.rs`、
  `src-tauri/tests/unit/{settings,appearance,geometry,window_geom,dock_state_view}.rs`、
  `src/{types,api,App}.tsx`、`src/dock/{Dock,Dock.tsx,Glyph}.tsx`、`src/settings/{validate.ts,SettingsPanel.tsx}`、
  `src/settings/preview.tsx`、`src/shelf/*`、`e2e/*`、`src-tauri/icons/*`、README、设计规范 §16。
- `src-tauri/icons/` 下的 PNG/ICO 是二进制资产，提交时必须显式列出，不要 `git add -A`。

提交规则：

- 只 stage 明确路径；**不要** `git add dock-tauri` 或 `git add -A`。
- **不要**提交 `target/`、`dist/`、`src-tauri/binaries/monitor-*.exe`。
- **不要**把 `vitro/`、`dock-app/`、`Glassboard-main/`、`cedro-modern-dock-main/`、
  `macos-dock-for-windows-master/` 的改动带进本产品提交。
- 提交前跑一遍 §6 的四套测试。

## 12. 不可触碰 / 安全约束

- **不要运行 `vitro.exe`**。
- 不要修改 `D:\dockyh\vitro\`、`D:\dockyh\dock-app\`、`D:\dockyh\Glassboard-main\`。
- `D:\dockyh\cedro-modern-dock-main` 和 `D:\dockyh\macos-dock-for-windows-master` 是只读参考；
  后者是 Photoshop 素材包，不是程序。不要复制其代码/素材进产品。
- **不要安装 MSI**，除非用户明确要求；安装会修改用户机器。
- 设置文件位于用户 `%APPDATA%`；测试时如修改，先备份再逐项恢复。**当前文件是用户自己的
  状态（固定项顺序已被用户调整），不要用旧备份覆盖**。
- 没有提交/推送指令时不要自行 commit 或 push。

## 13. 接手检查清单

- [ ] 读设计规范 §14 / §15 / §16 和 Phase 3 / 4 / 5 计划顶部的实现状态。
- [ ] 跑通 Rust / Vitest / Playwright / pytest 四套测试。
- [ ] 启动 release exe 或 dev，手工做 §8 的六项验收（含拖动与 Shelf）。
- [ ] 确认没有残留进程、Vite 1420 端口未被占用异常。
- [ ] 查看 `git diff --stat` 与 `git status`，与负责人确认提交分组。
- [ ] 与用户确认 Acrylic 方案和是否继续拖拽/Shelf/Docklet。
- [ ] 若改动 settings schema，递增 `CURRENT_SCHEMA`、补迁移测试、同步 `validate.ts`。
- [ ] 若改动几何/输入，补 `geometry.rs` / `hit_test.rs` 纯函数测试并重新做手工穿透验收。
- [ ] 重新打包后复测两个内存门禁和空闲 CPU。

## 14. 参考

- 设计规范：`docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`（§14 Phase 3，§15 Phase 4，§16 Phase 5）
- Phase 1 计划：`docs/superpowers/plans/2026-10-26-tauri-python-dock-implementation.md`
- Phase 2 计划：`docs/superpowers/plans/2026-10-26-dock-tauri-phase2.md`（T18 Acrylic 三方案）
- Phase 3 计划：`docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md`
- Phase 4 计划：`docs/superpowers/plans/2026-10-28-dock-tauri-phase4-drag.md`
- Phase 5 计划：`docs/superpowers/plans/2026-10-29-dock-tauri-phase5-shelf.md`
- 美术资源包：`dock-tauri/design-art/`（README + VISUAL-SPEC；只读参考，按需取用）
- README：`dock-tauri/README.md`
