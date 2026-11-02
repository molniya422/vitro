# Vitro 整合蓝图 v2 — 取精华·去糟粕·重新整合

> 2026-10-24。本文件是全项目唯一权威整合文档:盘点所有资源,对 NeXuS / Glassboard / macOS 规格 / HTML 原型四方做逐项取舍,定义 vitro v2 的模块结构与特性矩阵。PLAN.md 保留为里程碑日志,后续实现以本文为准。

## 1. 资源清单与处置

| 资源 | 内容 | 处置 |
|---|---|---|
| `D:\rea\nexus-dock-behavior-spec.md` | NeXuS 逆向行为规格 R1-R12(Frida 证据) | **保留引用** — 行为对拍基准 |
| `D:\rea\nexus-*.js/.py/.json` | NeXuS 逆向取证脚本与产物 | 保留(证据链,不再动) |
| `D:\dockyh\Glassboard-main` | Tauri+Svelte 屏幕标注工具(参考项目) | **只读借鉴**:Win32 窗口层/托盘/偏好/更新骨架;不引入其运行时 |
| `D:\dockyh\transitions.dev-main` | 动效片段库 | 已冻结为 motion-tokens.toml,留档 |
| `D:\dockyh\macos-dock-spec.md` + `macos-parity.toml` | macOS Sonoma/Tahoe Dock 设计规格与参数 | **合并**进本文 §4,原文件留档 |
| `D:\dockyh\effects-catalog.toml` / `motion-tokens.toml` | 效果目录 / 运动令牌 | 保留,渲染层引用 |
| `D:\dockyh\competitive-research-2026-10.md` | 竞品调研(mydockfinder/Seelen 等) | 保留;Seelen 为 AGPL,**只看行为不看代码** |
| `D:\dockyh\vitro\src\` | 原生实现 M6(main 843 / lpad 388 / plugin 109 / theme 43) | **v2 就地升级**,见 §5 |
| `D:\dockyh\vitro\selftest\` | Frida 自证(0 轮询证据 JSON) | 保留,每里程碑复跑 |
| HTML 原型 5 件(visualizations 目录) | M0 视觉规格 | 保留为视觉验收基准,不发布 |
| `ui-ux-pro-max-skill-main.zip` 等 zip | 原始压缩包 | **糟粕**:已解压的留 zip 无用,不参与构建 |

## 2. NeXuS 取舍(R1-R12 对照)

**取(精华)**

| 规格 | 行为 | vitro 落点 |
|---|---|---|
| R2 | falloff 放大(峰值 1.6/指数 1.6/邻域) | ✅ 弹簧缓动版(M6) |
| R3 | tooltip 停留显示 | ✅ 200ms dwell |
| R4 | 点击启动 + 运行句柄跟踪 | ✅ ShellExecuteEx + HANDLE |
| R5 | 启动弹跳/运行指示点 | ✅ 呼吸点(含 CPU 呼吸频率) |
| R6 | **拖放加图标** | ❌ M6 重写时丢失 → **v2 恢复**(WM_DROPFILES) |
| R7 | 右键菜单(打开/移除/…) | ✅ v2 扩展 |
| **R9** | **自动隐藏 + 屏幕底边召唤** | ❌ 未做 → **v2 核心新增**(零轮询版) |
| R11 | 配置持久化 + 去重清洗 | ✅ dock.toml |
| R1 | 屏幕定位(底部居中) | ✅ v2 改**宽度自适应**(修 900px 空板问题) |

**去(糟粕)**:R6b 轮询架构(GetCursorPos 40-68 次/s、常驻定时器群)——vitre 用钩子事件驱动替代,已用同一套 Frida 钩子对拍证实归零;R8 子码头(v2 缓);R10 皮肤/字体/透明度开关大杂烩(收敛为 theme.toml 五参数);GDI 渲染(换 DComp)。

## 3. Glassboard 取舍

**取(精华)**

| 机制 | Glassboard 做法 | vitro 落点 |
|---|---|---|
| 呼出范式 | 快捷键进入 overlay,再按清场,"draw then get back to work" | dock 底边召唤(R9)+ Ctrl+Alt+D 抽屉,同一种"即用即隐" |
| 窗口管理层 | `windows/{z_order,geometry,setup,toolbar}.rs` 纯 Win32:多窗 z 序、按显示器几何、工具窗属性 | 对应 vitro 的 TOPMOST+TOOLWINDOW+NOREDIRECTIONBITMAP 组合;其 geometry.rs 的按显示器布局思路进 backlog(多显示器) |
| 托盘 | `tray.rs` + 菜单退出 | **v2 新增**托盘图标(显隐 Dock/抽屉/主题/退出) |
| 偏好系统 | `preferences.rs`(673 行)分层默认值+迁移 | 借思路:theme.toml/dock.toml 已是平铺版,分层默认进 backlog |
| 更新器 | `updates.rs` 签名校验的 GitHub release 自更新 | backlog(v1 先手动发布,不联网) |
| 隐私立场 | 无账号无服务器,数据不出本机 | **采纳为项目原则**,写进 README |

**去(糟粕)**:Tauri/WebView 运行时(内存 70-120MB,vitro 硬预算 ≤30MB 的对立面)、Svelte UI 层、标注功能域(与 dock 无关)、每日联网检查更新(v2 保持零联网)。

## 4. macOS 规格 + 原型取舍(视觉验收基准)

取:玻璃面板(亚克力+内高光+底缘内影+1px 描边)、图标倒影(PushLayer 渐变遮罩镜像)、squircle 磁贴底衬(低透明度统一非规则 Windows 图标)、弹簧放大 cubic-bezier(0.34,3.85,0.64,1) 等价缓动、Launchpad 抽屉、falloff 0.45/1.6。
去:纯 CSS 手段(filter/backdrop-filter)——原生等价必须 DComp/DWM 实现;过度光效(只保留内高光+描边)。

**v2 视觉修复清单**(针对用户两次否决的截图):
1. 宽度自适应:`dock_w = n·icon + (n-1)·gap + 48`,上限屏宽-160 → 消灭两侧空板
2. 亚克力真正生效:`backdrop_ok` 时 tint α 降至 ~0.26(模糊/噪声由系统提供),失败才用 0.62 浓 tint
3. 1px 白描边 + 顶缘高光增强立体感;图标底衬 squircle(白 5%)
4. 图标 56→60,间距 14→16,面板 84→92

## 5. vitro v2 模块结构(整合后)

```
vitro/src/
├── main.rs    窗口/管线/渲染/交互/托盘/自动隐藏 (核心,~1000 行)
├── lpad.rs    Launchpad 抽屉启动器(搜索/命令/计算三模式)
├── plugin.rs  boa_engine 插件沙箱(api.set/every/log)
└── theme.rs   主题五参数 + 热重载
```

| 数据流 | 说明 |
|---|---|
| 输入 | WH_MOUSE_LL(全局,8ms 节流)→ PostMessage → 渲染;热键 RegisterHotKey |
| 召唤/隐藏 | 钩子判定底边 3px 或 dock±200 邻域;离开 700ms → 滑出屏外(33ms 动画),零轮询 |
| 渲染 | D3D11→D2D→DComp,只在与 dock 交互时重绘;空闲 0 定时器 |
| 持久化 | ~/.vitro/dock.toml(items/plugins/autohide) + theme.toml |

## 6. 特性矩阵(来源 → 状态)

| 特性 | 来源 | v2 状态 |
|---|---|---|
| 零轮询事件驱动 | 自研(反 NeXuS R6b) | ✅ Frida 实证 |
| falloff 弹簧放大 | NeXuS R2 / macOS | ✅ |
| 高清图标提取(96px 三档) | macOS 质感 | ✅ |
| **宽度自适应** | macOS | 🔧 本次 |
| **底边召唤 + 自动隐藏** | NeXuS R9 / Glassboard 范式 | 🔧 本次 |
| **拖放加图标** | NeXuS R6 | 🔧 本次(恢复) |
| **托盘** | Glassboard | 🔧 本次 |
| 玻璃质感修正 | macOS 规格 | 🔧 本次 |
| 抽屉启动器 | macOS Launchpad | ✅ M3 |
| 插件沙箱 | QuickJS 精神 | ✅ M4 |
| 主题/呼吸/电池 | 自研创新 | ✅ M5 |
| 倒影 | macOS | ✅ M6 |
| 子码头/多显示器/更新器 | NeXuS R8 / Glassboard | backlog |

## 7. 明确不做(糟粕清单)

轮询式输入(GetCursorPos/timer 驱动渲染)、WebView/Tauri 运行时、皮肤打包格式、联网更新(v1)、Seelen 代码(AGPL,仅行为观察)、多进程渲染服务。
