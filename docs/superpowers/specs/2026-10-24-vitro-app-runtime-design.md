# Vitro AppRuntime 渐进重构设计

## 目标

将 Vitro 从多个 static mut 全局和跨模块副作用构成的原生程序，逐步迁移为单线程事件驱动的 AppRuntime。迁移期间保持 Dock 可启动、配置兼容、视觉/交互能力可逐步验证，并降低状态共享、窗口销毁、定时器生命周期和插件上下文相关的故障风险。

## 保持不变的产品约束

- Windows 原生 Rust + D3D11/D2D/DComp/DWM；不引入 WebView、Tauri 或 AGPL 代码。
- 插件 JS 引擎固定 boa 0.17，不升级到 boa 0.18。
- Dock HWND 与交换链尺寸只在启动时确定；窗口运行期间不调整交换链尺寸。
- 鼠标输入由 WH_MOUSE_LL 邻域门控后投递消息；禁止常驻光标轮询。
- 33ms 动画 timer 只在弹簧、滑入滑出、运行呼吸等确有动画时存在；静止后撤销。
- RAM 预算 ≤30MB，空闲 CPU 接近 0%。
- 用户配置继续兼容 ~/.vitro/dock.toml 和 theme.toml，所有写入走备份/自愈存储层。
- Dock 保持可关闭；重构和构建阶段不自动启动它。

## 视觉与交互参考

`docs/research/00-SUMMARY.md`、`macos-dock-spec.md` 与 `nexus-dock-behavior-spec.md` 决定产品规格；HTML 原型只提供比对材料，不是实现指令。`vitro-macos-parity.html` 用于核对 Sonoma 材质层次和几何关系，原生端仍按冻结尺寸并使用 DWM/D2D；`vitro-m05.html` 和 `vitro-effects-gallery.html` 用于状态与动效样例；`vitro-nexus-full.html` 用于补全拖放、菜单、边缘触发等交互清单。不要照抄 CSS 模糊/噪点、舞台控件、模拟窗口预览或超出研究预算的无限动画。当前迁移阶段只修架构边界，不同时改视觉参数。

## 现状与问题边界

当前 main.rs 兼任窗口/GPU/图标/插件装配和消息循环；state.rs 通过 static mut STATE 暴露所有 HWND、D2D 资源、进程句柄、配置与动画状态；dockwin.rs 同时处理窗口消息、输入、tooltip、菜单、拖放、配置保存、主题重载和整帧绘制。input.rs、tray.rs、lpad.rs 直接修改该全局状态；plugin.rs 使用全局插件显示值，加载完成后保留不住执行上下文。纯逻辑因此与 Win32/D2D 耦合，生命周期与行为缺少小范围测试。

## 目标模块边界

    src/
      core/
        model.rs       DockItem、PluginSpec、Theme、TileId、布局值对象
        state.rs       AppState 与不可变 RenderSnapshot
        event.rs       InputEvent、PlatformEvent、TimerEvent、Action
        reducer.rs     纯状态转移
        geometry.rs    屏幕/窗口几何、药丸布局、命中测试
        motion.rs      放大、弹簧、隐藏状态和 timer 活跃条件
      runtime/
        app.rs         AppRuntime 所有权与事件分发
        shutdown.rs    单次、幂等的退出清理
      platform/windows/
        window.rs      HWND 类注册、WndProc、消息到平台事件转换
        input.rs       鼠标钩子、RegisterHotKey、系统电源事件
        shell.rs       ShellExecute、进程通知/状态、图标提取
        tray.rs        托盘注册与菜单事件
      render/d2d/
        renderer.rs    RenderSnapshot 到 D2D/DComp 绘制
        resources.rs   GPU、画刷、位图、字体 RAII 资源
      persistence/
        config.rs      TOML 解析、验证、备份、自愈和原子写入
      launcher/
        model.rs       查询、UTF-16 输入、命令/计算模式纯逻辑
        window.rs      启动器窗口输入与展示
      plugins/
        runtime.rs     每插件独立 boa Context 与定时任务

这些路径是目标边界；迁移时可先保留当前文件作为兼容门面，再逐步移动实现，不能为了目录形状一次性重写已稳定的 GPU 代码。

## 数据流与所有权

唯一的 AppRuntime 在 UI 线程持有 AppState、窗口句柄、平台服务和渲染器。渐进迁移期间，旧的 Win32 State 暂时作为兼容适配器由 AppRuntime 持有；在 HWND、低级钩子和托盘回调都改为从运行时上下文取状态前，不宣称 static mut 已移除。WndProc 与低级钩子只把外部输入转成 PlatformEvent，不直接保存配置或绘制。AppRuntime 收到事件后调用纯 reducer::reduce(&mut state, event) -> Vec<Action>；平台 action 执行 Shell/定时器/配置 I/O，状态变化后生成只读 RenderSnapshot，交给 D2D renderer。

COM/D2D 对象留在 UI 线程的渲染器所有权内。领域状态只保存可克隆值与稳定 ID，不持有 HWND、D2D 接口、HANDLE 或 &'static mut。运行中进程句柄归 platform/windows/shell.rs 的 ProcessMonitor，以事件把启动/退出/CPU 样本送回 runtime。

动画与延迟任务通过 reducer 返回显式 Timer action：TimerId 区分 Animation、TooltipDwell、AutoHide、LauncherDwell 和 (PluginId, PluginTimerId)；TimerSpec 指定间隔及 OneShot/Repeating，CancelTimer 取消单个 timer，CancelAllTimers 用于关闭。Windows 适配层把这些 action 映射到 SetTimer/KillTimer，并按 ID 去重。无弹簧结算、无滑动、无可见运行呼吸时必须撤销 Animation；tooltip/launcher/auto-hide dwell 使用 OneShot；关闭时撤销全部 timer，插件卸载时取消该插件的所有 timer。只在 hover 的静态目标已收敛时，不继续渲染。

低级鼠标钩子保留现有 ±200 px 邻域/屏幕底边召唤区门控、8 ms 内部采样节流和离开补发。进入/离开门控区域的边界事件立即投递，避免短暂跨越完全漏掉；只有区域内连续样本受 8 ms 节流。它不读取或修改 AppState：运行时只发布一个小型原子 HookSnapshot（Dock hwnd、药丸屏幕矩形、屏幕高度、autohide）；钩子以该快照决定是否投递坐标事件，邻域外仍是零消息、零渲染。窗口移动/药丸几何或 autohide 变化后由 UI 线程更新快照。该快照是输入适配器缓存，不是第二份领域状态。

插件配置仍由 TOML 保存。每个插件有自己的 boa 0.17 Context、显示值和 interval 回调表，并保留现有 api.set、api.every(ms, fn)、api.log 脚本接口。api.set 产生领域事件；api.every(ms, fn) 创建带 PluginId/PluginTimerId 的 repeating timer action，间隔限制在 100 ms 至 24 小时；插件移除或关闭时撤销其 timer。运行回调只在允许的 UI 线程调度，不允许插件直接访问 AppState 或 Win32 指针。插件错误被记录并隔离到对应实例。

## 生命周期与错误处理

关闭请求、退出菜单、Esc 和系统关闭进入一个 Shutdown 事件。Shutdown 只能执行一次：保存配置、移除托盘、卸载钩子、关闭进程监控、撤销 timer、销毁 tooltip/launcher/dock HWND，再由终止窗口消息投递 WM_QUIT。WM_DESTROY 仅确认窗口已销毁，不反向调用销毁流程。

平台边界的操作返回 Result 或显式事件，不在资源创建失败时随意 unwrap。配置损坏必须保留原始字节到 `dock.toml.corrupt` 或 `theme.toml.corrupt`（遇到同名文件时选择不覆盖的唯一后缀）；保存前将最后一份有效文件复制为 `.toml.bak`，再写同目录临时文件并原子替换目标。配置路径清洗拒绝控制字符、NUL 和设备路径前缀，不静默重写用户路径。读取/写入失败向日志报告，不能悄悄覆盖用户素材。

## 迁移兼容策略

按可回退的小步迁移，现有 dock.toml、theme.toml 字段名与默认行为保持兼容。先将纯几何、动效、启动器输入等可测逻辑移出平台代码；再引入显式 AppState/事件/reducer 和 Runtime façade；随后把 HWND 消息输入、鼠标 HookSnapshot、Shell、持久化、渲染逐一接到边界上；最后删除 static mut STATE、static mut LP、static mut PLUGIN_*。每一步都保持 release 构建有效，并避免同时改变视觉令牌和行为。

启动器和插件命令输出、R8 子码头、多显示器、OLE 拖出是独立功能，不属于首轮架构迁移。它们等稳定的 event/runtime API 建立后再分别规划，避免把新功能和基础迁移混在同一变更里。

## 验收

1. cargo test --manifest-path D:\dockyh\vitro\Cargo.toml：纯 reducer、坐标、动效、timer policy、配置恢复和插件调度测试通过。
2. cargo build --release --manifest-path D:\dockyh\vitro\Cargo.toml：Windows release 构建通过。
3. 静态检查确认没有领域模块依赖 windows crate；没有 static mut STATE/LP/PLUGIN_*；WndProc 不直接保存配置或调用 renderer。
4. 启动运行时的 Frida selftest：空闲 GetCursorPos/轮询调用为 0；无动画时没有常驻 timer。
5. 8 秒采样：空闲 CPU <1%，RAM ≤30MB。
6. 用户明确要求保持停止时，验证不得通过启动 Dock 完成；准备运行时截图时先提交验证步骤给用户选择，不擅自启动。
7. 用户允许启动进行最终视觉验收后，用项目的 PrintWindow(PW_RENDERFULLCONTENT) 脚本验证 DComp 内容；CopyFromScreen/BitBlt 不作为验收依据。

## 首个实施阶段范围与验收

首阶段先建立真实的 UI 线程所有权：把现有 Win32 `State` 原样收进稳定地址的 `Box<AppRuntime>`，由 Dock/tooltip HWND 的 `GWLP_USERDATA` 找到它；窗口过程只做消息转换，业务处理通过短作用域访问运行时，删除 `static mut STATE` 与 `s()`。低级鼠标钩子没有窗口 userdata，因此只保留小型原子 `HookSnapshot` 做邻域门控、8 ms 节流和离开补发；自动隐藏、滑动与 timer 决策移到 Dock 窗口消息处理。`TrackPopupMenu` 的嵌套消息循环和 `DestroyWindow` 同步触发的 `WM_DESTROY` 期间不得持有运行时的可变借用；需要先复制 HWND/菜单输入，再调用 Win32 API。

此阶段不拆 GPU/COM 资源，不声称纯领域 reducer、插件重构或所有 static mut 已完成。`LP`、`PLUGIN_*` 暂时留在各自模块，后续分别迁移。首阶段测试覆盖 HookSnapshot 的门控/节流边界与自动隐藏事件决策；验收为 `cargo test`、`cargo build --release`、静态检查确认 `STATE`/`s()` 删除、Dock 保持关闭。CPU/RAM、Frida hook 和视觉截图属于运行时全面接线后的验收，不在本阶段伪称已验证。

后续按独立实施计划拆为：纯 `AppState`/事件/reducer 与稳定 TileId、坐标和命中测试；持久化适配器；Shell/进程监控；D2D `RenderSnapshot`；launcher/lpad；插件 Runtime/定时器；最后清除剩余 `static mut LP/PLUGIN_*` 并进行完整验收。进程监控不得另起常驻轮询：沿用窗口动画 timer 只在至少一个进程图标可见且需要更新时采样；停止动画、隐藏 Dock 或进入低电量策略时降频/暂停，并在无活跃磁贴时撤销采样 timer。
