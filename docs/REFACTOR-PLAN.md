# Vitro Dock 重构计划

## 目标

把当前“能运行但由多个 static mut 全局状态拼接而成”的 v3 原型，重构成单线程事件循环驱动的分层应用，同时保留已验证的产品约束：

- 固定画布在启动时确定，创建后不调整 HWND 或交换链尺寸。
- 鼠标输入由低级钩子转换为窗口消息；没有常驻光标轮询。
- 动画定时器只在弹簧、滑入滑出或运行指示确实活跃时存在。
- RAM 不超过 30 MB，空闲 CPU 接近 0%。
- 配置通过 config.rs 的备份与自愈管线读写。
- 插件继续使用 boa 0.17；不引入 WebView、AGPL 代码或新的常驻服务。

## 现状问题

main.rs 同时装配窗口、DWM、GPU、图标、插件、托盘和消息循环；state.rs 通过
static mut STATE 暴露所有可变状态；dockwin.rs 同时处理窗口过程、菜单、拖放、tooltip、
主题、进程句柄和整帧绘制；lpad.rs 还维护第二套独立的 GPU 管线。这样一来，输入回调可以
直接触发渲染和持久化，纯逻辑也无法测试。

## 分阶段结构

### Phase 1：AppRuntime 所有权

先把现有含 GPU/Win32 资源的 `State` 原样移入稳定地址的 `Box<AppRuntime>`，由 Dock 与 tooltip 窗口的 `GWLP_USERDATA` 找到它，逐步让回调显式接收运行时上下文，删除 `static mut STATE` / `s()`。低级钩子保留一个小型原子 HookSnapshot，并把门控、节流、边界转换策略留在无 Win32 依赖的 `core/input.rs` 测试；钩子只投递鼠标样本，自动隐藏与动画 timer 决策回到 UI 消息处理。嵌套菜单循环与 `DestroyWindow` 调用期间不能持有运行时的可变借用。

此阶段不拆 COM/D2D 资源，不改变视觉令牌，也不迁移 `LP` 或插件静态值；Dock 保持关闭，验收只覆盖测试、release 构建和 `STATE`/`s()` 已删除。

### Phase 2：领域核心

新增无 Win32 依赖的纯 Rust 模块：

- launcher_model.rs：查询模式、UTF-16 输入累积、命令输出截断。
- core/model.rs：磁贴、插件、主题和布局快照。
- core/motion.rs：放大、自动隐藏和 reduced-motion 的纯计算。
- core/reducer.rs：InputEvent/TimerEvent 到 AppState 的显式状态转换。

这些模块只接受值和事件，单元测试覆盖坐标转换、命中测试、模式解析、代理输入和计时器活跃条件。

### Phase 3：平台适配

- platform/windows/input.rs 只负责 WH_MOUSE_LL、热键和 WM 消息。
- platform/windows/shell.rs 负责 ShellExecute、进程句柄和图标提取。
- platform/windows/persist.rs 包装 config.rs，所有 I/O 返回 Result。
- 回调只发送事件，不直接调用 render_frame()、save_state() 或修改领域状态。

### Phase 4：渲染端口

render/d2d.rs 接受只读 RenderSnapshot，持有 GPU 资源、画刷和位图；
渲染器不读取进程句柄、不管理定时器、不执行配置写入。固定画布和 DComp 透明穿透继续保留。

### Phase 5：运行时编排

单线程 AppRuntime 持有领域状态、平台端口和渲染器，执行：

PlatformEvent -> reducer -> RenderSnapshot -> renderer

退出只走一次 Shutdown 事件，窗口过程不再递归 DestroyWindow。插件宿主拥有独立的
PluginRuntime 和每插件上下文，插件更新通过事件返回。

### Phase 6：能力恢复顺序

1. 静态磁贴、主题和固定画布。
2. 放大、tooltip、自动隐藏。
3. Shell 启动和运行指示。
4. 拖入、右键菜单和持久化。
5. 启动器和 Unicode 输入。
6. 插件定时更新。
7. 子码头、多显示器、OLE 拖出和窗口缩略图。

每个阶段必须通过 cargo test、cargo build --release、8 秒 CPU/RAM 采样、PrintWindow
视觉检查和 selftest/run-selftest.py。

## 参考资料优先级

- `docs/research/00-SUMMARY.md`、`macos-dock-spec.md` 和 `nexus-dock-behavior-spec.md` 是决策依据。
- `docs/prototypes/vitro-macos-parity.html` 用于比对材质层次和几何关系；落地继续使用 DWM Acrylic、D2D 与位图噪点，不复制网页滤镜。
- `vitro-m05.html` 和 `vitro-effects-gallery.html` 提供状态/动效样例，不覆盖研究冻结的尺寸与克制动效预算。
- `vitro-nexus-full.html` 补足拖放、菜单和边缘触发的交互清单；模拟舞台控件、未经证实的行为及网页玻璃参数不进入原生常驻 Dock。

架构迁移不照搬网页演示的滤镜和舞台控件；本轮同步做了研究冻结的几何材质校准。后续视觉调整仍以双主题、4px 网格、无外描边/外投影、固定画布、reduced-motion 与静止零 timer 为验收条件。

## 本次落地

- 停止正在运行的 vitro.exe。
- 完成 AppRuntime 第一阶段所有权迁移：旧 Win32 `State` 收入稳定 `Box<AppRuntime>`，Dock/tooltip 通过 `GWLP_USERDATA` 找到它，删除 `static mut STATE` 与 `s()`。
- Hook 仅读取路由快照并投递坐标；Dock 与托盘菜单在嵌套弹出循环返回后检查关闭 gate，避免 teardown 后继续执行命令。
- 启动阶段 hook 或启动器初始化失败时走统一 Dock teardown；启动器局部 GPU 初始化失败会销毁临时 HWND。
- 新增 launcher_model.rs，把查询解析和 UTF-16 输入从 Win32 窗口过程剥离，并以单元测试锁定行为。
- 将启动器的查询过滤和 WM_CHAR 路径接到纯模型层，中文和代理对不再被低 8 位截断。
- 修正 KillTimer 返回值处理，降低事件层噪音。
- 保留工作区已有的 Sonoma 噪点/高光 dirty patch，后续以 PrintWindow 复核后再单独提交。

## Phase 2 进度：Dock 行为 reducer 切片

- 已新增无平台依赖的 `core/model.rs`、`core/event.rs` 与 `core/reducer.rs`；`AppRuntime` 持有可克隆的 `BehaviorState`，WndProc 的指针、tooltip dwell、自动隐藏、显示和切换决定都通过纯 reducer 转移。
- reducer 返回 timer、tooltip、配置保存、HookSnapshot 发布和 render effects，由 Dock UI adapter 调用 Win32 与现有绘制/配置层执行；旧 GPU-rich `State` 不再重复持有 cursor、hover、tooltip、autohide、hide-armed 或 slide-target 状态，当前 slide 插值仍留在 renderer adapter。
- 磁贴用 `TileId` 标识；因为菜单允许删除磁贴，索引会移动，故 hover/tooltip 身份不能安全依赖向量索引。添加、删除路径已在 adapter 中分配/清理 ID。
- 修正了跨磁贴移动时旧 tooltip 残留的问题；菜单跨 `TrackPopupMenu` 嵌套循环保存 `TileId`，返回后重新解析索引，避免重入期间误操作相邻磁贴。
- 按 `macos-dock-spec.md` 校准了当前材质几何：面板 68px、图标 48px、间距 8px、左右内边距 16px、圆角为面板高度的一半；固定画布扩展为 320px，为 R8 子码头保留上方缓冲，并保留 32px 倒影、噪点和 1px 内高光/内影。
- 退出路径在销毁窗口前注销热键、关闭拖放接收、撤销低级钩子并清理定时器；Dock 进程保持停止，视觉运行验收待后续用户批准。
- 插件显示文本改为每次 Boa 加载独立的上下文，不再通过进程级 `PLUGIN_TEXT` 串值；插件正文和小徽章的 DirectWrite 格式由 `State` 持有并随 AppRuntime 释放。
- 目前 `cargo test` 通过 43 项，`cargo build --release` 通过；没有启动 Dock。lpad 启动器状态（包括窗口、GPU 管线、输入缓冲和 DirectWrite 格式）已收入 `AppRuntime.launcher`，并删除 `LP/LP_FMT` 两个进程级静态状态；启动器窗口通过 `GWLP_USERDATA` 回到运行时。视觉切片修正了运行指示点越界、放大图标底边漂移和插件分区线，并让命中区域跟随放大后的图标尺寸。新增基础的 Dock 内拖动重排，拖动中的图标会轻微抬升并放大，当前目标磁贴会显示 accent 插槽圈作为松开提示，同时同步维护磁贴关联的进程句柄和动画/CPU 状态。R8 子码头现在使用固定透明画布上方的玻璃区域：目录磁贴悬停展开、缩放入场、反向关闭动效、子项高亮、稳定 tooltip 身份/定位和 Shell 启动均已接通；动画未完成时不接受子项命中，透明缓冲区对后方窗口返回穿透命中；OLE 拖出仍未实现。此切片仍不包含 renderer/GPU 端口化或完整插件运行时拆分。

本切片之后仍需迁移完整的 DockItem/Plugin/Theme `AppState` 与 RenderSnapshot；把像素命中/布局的纯几何 API 纳入 core；为 timer 生命周期与配置持久化建立独立 port/adapter；迁移 Shell/进程监控和插件运行时；补足 OLE 拖出、多显示器布局、异步子码头目录加载和完整材质验收；最后完成 CPU/RAM、Frida 与用户批准的视觉验收。Phase 2 其余模块和后续平台/渲染阶段均未完成。

## 暂缓项

命令输出捕获、api.every 调度、插件 flyout 和多显示器适配属于后续阶段；
它们依赖新的事件循环和生命周期边界，不在旧的全局状态上继续堆叠。

