# Dock App 资源、设计与实现细化规格

**日期：** 2026-10-25  
**状态：** 设计细化，供实施前审阅  
**适用产品：** `D:\dockyh\dock-app` 独立 Windows 桌面增强套件

这份规格把“可以直接调用”“只能参考”“必须自己写”“需要联网核验”分开，并把视觉语言、功能闭环、创新点和验收指标固定下来。它补充而不替代 [桌面套件设计](D:/dockyh/docs/superpowers/specs/2026-10-25-desktop-suite-design.md) 与 [实施计划](D:/dockyh/docs/superpowers/plans/2026-10-25-desktop-suite-implementation.md)。

## 1. 固定边界

首发产品是独立的 Windows 桌面增强套件，包含 Dock、Launchpad、Shelf、窗口与任务管理、受控系统托盘、Docklet 插件和设置。Windows 原生任务栏继续由 Explorer 管理，Dock 与任务栏共存；Dock 自己可以自动隐藏，但不注入 Explorer、不替换 Shell、不修改 DWM。

`Glassboard-main` 保留为独立的标注与截图产品，不进入 Dock 发布包。`vitro` 保留为 legacy reference，不作为新产品依赖。GlassDesk、Nexus、BitDock、MyDockFinder、Rainmeter、Seelen 和 Übersicht 只提供公开行为、架构和设计线索，不能复制代码、品牌、私有资源或受限制实现。

首发目标平台固定为 Windows 11 x64，覆盖 23H2、24H2 及之后仍受支持的版本；启动时做能力探测，缺少 DWM 背板、缩略图或特定托盘能力时使用降级实现并显示可解释状态。

## 2. 资源调用矩阵

### 2.1 可以直接调用的资源

| 资源 | 直接用途 | 约束 | 验证方式 |
|---|---|---|---|
| `D:\dockyh\docs\specs\effects-catalog.toml` | 效果槽、预设、性能层级的只读输入 | 不在运行时修改原文件；复制到 `dock-app/assets/tokens/` 时保留来源 | TOML schema 校验；11 槽、44 预设逐项映射测试 |
| `D:\dockyh\docs\specs\motion-tokens.toml` | signature/spring、tooltip、panel 和全局守卫 | 参数是本项目规格，不把 HTML 动画代码带入原生端 | token golden test；缩减动效和省电档测试 |
| `D:\dockyh\docs\specs\macos-parity.toml` | Sonoma/Tahoe 材质、几何、状态和无障碍参数 | 参数进入自有 renderer，不复制 Apple 代码或品牌素材 | 100/125/150% DPI 截图和几何误差测试 |
| `D:\dockyh\docs\research\*.md` | 需求、行为、设计和竞品验收依据 | 只读；每条要求必须链接到测试或验收项 | 需求—测试覆盖表 |
| `D:\dockyh\docs\prototypes\vitro-*.html` | 本地视觉基准、交互录制、状态对照 | 不打包、不嵌入 WebView、不复制 DOM/JS 运行时 | 浏览器截图与原生 `PrintWindow` 对拍；人工审美评审 |
| `D:\dockyh\vitro\assets\icons\*.png` | 原型和内部视觉对照 | 当前为 96×96 PNG；在来源和商标检查完成前不能作为发布资产 | SHA-256、来源清单、商标检查、DPI 视觉审查 |
| Windows SDK / Win32 / Shell32 / DWM / Direct2D / DirectWrite / DirectComposition / WIC / UI Automation | 平台真实数据、原生窗口、绘制和无障碍 | 只通过自有 Rust adapter 使用；不能把 HWND 泄露给 Docklet | 能力探测、平台集成测试、UIA 验收 |
| Rust crates `windows`、`serde`、`toml`、`uuid`、`thiserror` | 类型安全绑定、序列化和错误模型 | 锁定版本并记录许可证；不引入 WebView 作为 Dock runtime | `cargo deny`/许可证清单、离线构建 |

Glassboard 的 MIT 许可证允许在法律上复制代码，但产品边界明确禁止把 Glassboard 代码并入 Dock 发布包。Glassboard 只能作为用户已有行为和错误处理的背景资料；发布包只包含独立 Dock App。

### 2.2 只能提取规格或算法，必须重新实现

| 来源 | 可提取内容 | 禁止事项 | 新实现落点 |
|---|---|---|---|
| `vitro/src/core/{geometry,input,motion,model,reducer,event}.rs` | 几何公式、事件语义、状态迁移、测试意图 | 不复制实现、模块结构、全局状态或旧类型 | `dock-core` 的纯函数和 reducer |
| `vitro/src/{gpu.rs,icons.rs,runtime.rs}` | D2D/DComp 生命周期、Shell 图标回退、shutdown gate 思路 | 不引用旧 crate，不复用 static mut、裸指针和旧窗口代码 | `windows-platform`、`native-ui`、`shell-runtime` |
| GlassDesk 1.4.0 | Core/Platform/AppServices/Dock/Launcher/Settings/UI 分层、行为边界和测试案例 | 不复制 PolyForm Noncommercial 新增代码、品牌、图标和材质 | 新 Rust workspace 的模块边界 |
| Nexus/BitDock/MyDockFinder | 多 Dock、子 Dock、Shelf、拖放、窗口预览、插件目录等公开行为 | 不逆向私有实现，不使用其商标和私有资源 | Nexus R1–R12 黑盒验收、自己的实现 |
| Rainmeter/XWidget/Übersicht/Seelen | 声明式 manifest、meter 词汇、插件目录和分发流程 | 不复制 Seelen AGPL 代码，不执行任意脚本 | `docklet-manifest`、QuickJS/helper sandbox |

### 2.3 必须自己写的资源和代码

- `dock-app` 全部领域模型、布局、reducer、菜单模型、持久化和迁移。
- Win32/COM/WinEvent/Monitor/DPI/AppUserModelID/窗口激活/拖放/回收站/托盘/任务栏 adapter。
- Win32 顶层窗口、D2D/DirectWrite 绘制、DComp 合成、WIC 图片解码、共享几何和命中测试。
- 11 个效果槽、44 个预设、macOS 克制档、弹性档、省电档和 reduced-motion 守卫。
- 图标 mask、noise texture、reflection mask、badge、running dot、tooltip、menu、flyout、preview 的发布级美术资源。
- Launchpad 的 Start Menu/AppUserModelID 扫描、搜索、分类、文件夹和分页。
- Shelf 的 fan/grid/list stack、文件拖放、Open With、Recycle Bin 错误回滚。
- WindowTracker、任务列表、窗口预览、受控系统托盘、时钟、快捷键和 UI Automation provider。
- Docklet manifest、签名/来源元数据、权限询问、资源限制、崩溃隔离、安装/启停/卸载和四种 Docklet 形态。
- 设置 schema、版本迁移、原子写、备份、自愈、导入导出、单实例 IPC、崩溃恢复和发布脚本。

### 2.4 必须联网核验的内容

联网只用于确认会变化的官方事实，不能用搜索结果替代产品设计：

1. Microsoft Learn：`SetWinEventHook`、WinEvents、UI Automation events、DPI、Shell icon extraction、OLE drag/drop、IFileOperation、DWM backdrop、DirectComposition 的最低系统版本和 fallback。
2. Apple 官方 HIG、macOS Dock 和 Liquid Glass 公告：只确认视觉原则，不把二手文章中的未来参数当成事实。
3. QuickJS 官方发布说明、许可证、字节码兼容性和资源限制能力；若无法可靠限制运行时，改用隔离 helper process。
4. `windows` crate 当前版本、feature 名称、许可证和 Rust stable 兼容性。
5. Windows 11 当前签名、MSIX、winget 和 GitHub Release 分发规则。
6. 天气、邮件、网络状态等数据源的 API、授权、速率限制和离线行为；首发必须允许 provider 缺失并显示 unavailable 状态。
7. 第三方图标、字体、噪点纹理和 Docklet 包的许可证；任何来源不清的素材不进入发布包。

已核对的官方技术入口包括 [Windows Shell 图标与图像列表](https://learn.microsoft.com/en-us/windows/win32/menurc/about-icons)、[SetWinEventHook](https://learn.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setwineventhook)、[UI Automation 事件](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-eventsforclients)、[DirectComposition 架构](https://learn.microsoft.com/en-us/windows/win32/directcomp/architecture-and-components) 和 [QuickJS 官方文档](https://bellard.org/quickjs/)。

## 3. 美术资源重构方案

### 3.1 资产目录

新仓库必须建立以下目录和清单：

```text
D:\dockyh\dock-app\assets\
  source\vitro-reference\       # 仅内部对照，默认不进发布包
  icons\vector\                 # 自绘 SVG/路径定义
  icons\raster\                 # 由 vector 生成的 16/24/32/48/64/96/128
  noise\noise-256.png           # 单色可平铺噪点
  materials\                    # 主题和折射 mask
  previews\                     # QA 截图，不进运行时
  manifest.json                  # source/license/author/hash/allowed_use
D:\dockyh\docs\PROVENANCE.md
D:\dockyh\docs\THIRD_PARTY_NOTICES.md
```

### 3.2 图标处理

现有 96×96 PNG 只用于原型对照和内部占位。发布前重新绘制一套 512×512 向量母版，再编译为 16、24、32、48、64、96、128 px。图标采用统一 20% 圆角 mask、相同视觉重心、相同底部基线和相同反射起始线；真实应用图标优先从 Windows Shell 按目标 DPI 提取，系统项和 Docklet 使用自有抽象图标。

每个图标必须通过三项审查：小尺寸仍能辨认、放大时边缘没有锯齿、与玻璃背景合成后主体对比度不低于 3:1。禁止把不同来源的图标直接混排，禁止为每个图标单独加不同阴影或外框。

### 3.3 材质与几何

- 默认深色 Tahoe/Sonoma：blur 22、黑色填充 25%、saturate 1.08、噪点 alpha 0.25%、顶部内高光 15%、底部内影 15%。
- 浅色档：blur 20、白色填充 18%、saturate 1.10、噪点 alpha 0.30%、顶部内高光 30%、底部内影 10%。
- 浮动档整体从工作区边缘抬升 24 px，增加折射边，但仍保持无外描边、无重外阴影。
- 默认 Dock 高度 86 px，半高圆角；水平 padding 16 px、垂直 padding 10 px、图标间隙 8 px；全部尺寸吸附 4 px 网格。
- 默认图标内容区 48 px，可调 16–128 px；运行点 4 px，离图标底部 6 px；badge 最小 16 px，右上偏移约 `(-4,-4)`。
- 文字使用 Segoe UI/Segoe UI Variable；Caption 12/16、Body 14/20、Body Large 18/24、Subtitle 20/28、Title 28/36；数字全部启用 tabular figures。

## 4. 视觉思路与原创设计点

整体风格定义为“安静的 Tahoe 深色玻璃”：材质有深度，图标和状态有信息密度，动效短而克制，任何装饰都不能抢过真实操作。视觉高级感来自材质层次、统一几何、留白和状态语义，不来自叠加发光、粗描边或持续粒子。

### 4.1 Adaptive Liquid Rail

Dock 主体保持统一玻璃药丸；边缘内高光根据当前背景亮度做小范围自适应，绝不改变命中区域。背景亮度变化只影响边缘高光和 tint，不改变图标颜色。目标是合成亮度偏差低于 8%，高光保持 1 px。

### 4.2 Focus Lens

鼠标附近才启用抛物线放大和局部 specular sweep，邻域外不做全屏重绘。放大通过共享几何同时驱动绘制、命中测试、tooltip 锚点和可访问矩形，避免视觉位置与点击位置分离。单次 hover 的 GPU 绘制预算低于 2 ms。

### 4.3 Semantic State Skin

运行、下载、通知、错误和进度都写入图标本体：运行点、badge、底部微型 sparkline 或 mini-dial。状态颜色只使用主题 accent、系统红、系统橙和系统绿；不新增浮动标签。每个 Docklet 磁贴只回答一个问题，更多信息进入 flyout。

### 4.4 Quiet Morph

Dock、Launchpad、Shelf、菜单和 flyout 共享稳定的元素 ID。打开时从触发项生长，关闭时逆向还原；动画被禁用时直接切换到同一终态，不改变布局和命中。首帧必须在 100 ms 内出现，关闭后几何误差低于 1 px。

### 4.5 Curated Density Dial

设置提供 density、variance、motion 三个独立旋钮。density 只改间距和尺寸，variance 只改材质明暗和 accent 强度，motion 只改时序和过冲；三者不能互相污染。任何主题仍遵守 4 px 网格和固定字阶。

### 4.6 Shelf as Spatial Memory

Shelf 的 fan/grid/list 不是三个皮肤，而是三种空间记忆：fan 适合少量最近文件，grid 适合浏览，list 适合精确操作。文件从触发点展开，拖入应用时显示 Open With 反馈，拖入回收站时显示可逆删除状态，失败必须恢复原位置。

## 5. 功能闭环和实现方式

每个功能都必须有真实输入、状态快照、平台动作、成功反馈、失败反馈和可访问操作。只有图标没有平台动作的按钮不得进入 UI。

| 模块 | 首发功能 | 真实数据/动作 | 实现方式 | 失败与降级 |
|---|---|---|---|---|
| Dock Shell | 四边停靠、每屏布局、放大、运行点、badge、菜单、重排、自动隐藏 | Monitor work area、DPI、App/Window snapshot、鼠标和 OLE drop | `dock-core` reducer + Win32 per-monitor window + D2D/DComp renderer | 监视器消失时迁移到主屏；定时器失败由状态机强制收起 |
| Launchpad | Start Menu 扫描、搜索、分类、最近、分页、拖入 Dock | AUMID、快捷方式、已安装应用目录 | 增量目录索引 + stable identity + fallback icon | 图标缺失显示系统 fallback；扫描失败保留上次索引和错误行 |
| Shelf | 文件夹、下载、最近文档、fan/grid/list stack、Open With、回收站 | Shell item、文件路径、IFileOperation/OLE | 栈模型与 DockLayout 分开持久化；拖拽事务松手提交 | 删除失败回滚；路径无效显示 unavailable |
| 窗口/任务 | 激活、恢复、最小化、关闭、窗口列表、缩略图、工作区/显示器 | WinEvent + EnumWindows reconcile、HWND 仅平台层持有 | 事件触发后 150 ms 合并枚举；renderer 只读快照 | 窗口消失发事件；权限不足禁用动作并说明原因 |
| 系统面 | 时钟、快捷键、受控托盘、任务栏共存 | 时间、Shell tray、taskbar bounds、注册热键 | 平台 provider + flyout；不伪造不支持的托盘动作 | 只显示可用动作；不支持项显示 unavailable |
| Docklet | icon+badge、60 点 sparkline、mini-dial、flyout | manifest 声明的系统数据和刷新周期 | schema 校验 + 权限询问 + QuickJS/helper sandbox + budget scheduler | 崩溃/超时自动禁用，宿主继续运行 |
| 设置 | 位置、尺寸、材质、效果、快捷键、权限、导入导出 | 版本化 TOML/JSON 文件 | tmp 写入、flush、backup、rename、schema migration | 主文件损坏读 `.bak`；两者损坏回默认并保留诊断 |

### 5.1 Docklet 权限固定表

首发只允许这些声明式权限：`read_clock`、`read_cpu`、`read_memory`、`read_battery`、`read_network_status`、`read_weather_provider`、`read_mail_provider`、`open_url`、`open_file`、`show_notification`。默认全部拒绝。

禁止 Docklet 直接访问 HWND、任意 Win32、任意文件路径、进程控制、剪贴板、注册表、网络 socket、DOM、`new Function` 和宿主 Rust 对象。每个实例默认 CPU 平均预算 0.2%、内存 8 MB、刷新周期不低于 1 秒；超预算连续三次自动暂停并给出恢复入口。

## 6. 动效实现和效果守卫

11 个效果槽固定为：magnify、launch、attention、drawer、menu、tooltip、reflection、icon tilt、confirm、badge counter、dock enter。效果目录中 44 个预设全部进入 schema 和设置 UI，但首发默认只显示经过性能审查的预设。

执行优先级固定为：

1. reduced motion：所有变形、弹跳、放大和折射变成不超过 200 ms 的 opacity/position fade；
2. battery/performance：每个槽回落到 `low_power`；
3. 用户 motion profile：macOS 克制、Vitro 弹性、省电；
4. 元素/槽冲突：同一元素同一槽只能有一个活动效果；launch 期间 magnify 只读，不写入新动画。

所有动画使用单调时钟和可取消 token；Dock 隐藏时停止 Docklet 采样和非必要动画；静止状态不保留永久高频 timer。

## 7. 指标和硬门槛

### 7.1 功能正确性

| 指标 | 目标 | 验证 |
|---|---:|---|
| Nexus R1–R12 覆盖率 | 100% | 自动化行为测试 + Windows 桌面验收 |
| 点击真实应用启动/激活成功率 | ≥99/100 次 | 固定应用矩阵，记录启动和激活错误码 |
| WinEvent 到窗口快照更新 | p95 ≤200 ms | 事件时间戳与 reducer 时间戳 |
| 自动隐藏 show/hide 时序 | show 300 ms、hide 500 ms；异常后 ≤1 s 收起 | 时间状态机测试、重复 10,000 次 |
| 拖拽重排提交/回滚 | 成功 ≥99%，写入失败 100% 回滚 | 模拟磁盘失败和 Explorer drop |
| 设置故障恢复 | 100/100 故障注入恢复最近有效快照 | 写入中断、进程终止、损坏主文件/备份 |
| Docklet 崩溃隔离 | 100% 宿主继续运行 | 崩溃、超时、恶意 manifest、资源超限 |

### 7.2 性能与稳定性

| 指标 | 目标 | 测量方式 |
|---|---:|---|
| 空闲 CPU | <1% | 60 秒无输入、无动画、无 Docklet 活跃采样 |
| 宿主私有内存 | <30 MB，不含 Docklet helper | Windows Performance Recorder/任务管理器交叉验证 |
| Hover 动画帧时间 | p95 ≤16.7 ms，GPU 绘制预算 <2 ms | ETW/DComp frame timing |
| 点击到视觉反馈 | p95 ≤50 ms | 输入时间戳到第一帧变化 |
| 冷启动到可点击 | p95 ≤400 ms；首次索引另计 | 启动埋点，排除系统冷启动极端值 |
| 监视器/DPI 变化恢复 | ≤250 ms | 插拔显示器、改变缩放、主屏切换 |
| 连续显示/隐藏稳定性 | 10,000 次无崩溃、无句柄持续增长 | UI 自动化循环和句柄采样 |
| 长时间运行 | 24 小时无崩溃；句柄增长 <2% | idle/hover/窗口变化混合脚本 |

### 7.3 视觉与无障碍

| 指标 | 目标 |
|---|---:|
| 关键几何位置误差 | 100% DPI ≤1 px；125/150% DPI ≤2 px |
| 普通文本对比度 | ≥4.5:1 |
| 大字和图形对比度 | ≥3:1 |
| 运行点尺寸 | 4 px ±0.5 px |
| tooltip 延迟 | 克制档 200 ms；弹性档 80 ms；设置可关闭 |
| reduced-motion 动画 | ≤200 ms，布局和命中不变 |
| UIA 覆盖 | 每个可操作项有名称、角色、矩形、Invoke/Keyboard 路径 |
| 截图对拍 | 关键状态结构差异 0；材质像素差异在预设容差内 |

### 7.4 安全和发布边界

- 新 crate 的源码引用扫描不得出现 `Glassboard-main`、旧 `vitro/src`、GlassDesk namespace、Seelen namespace 或未经批准的第三方资源路径。
- 发布包不得包含 Glassboard、Vitro runtime、未审计 PNG、未审计字体、未签名 Docklet 或开发诊断日志。
- 每个资产在 `assets/manifest.json` 中有 `source`、`license`、`author`、`hash`、`allowed_use` 和 `verified_at` 字段。
- 每个 Docklet 在安装前完成 manifest、签名/来源、权限和资源预算验证；失败安装不留下半成品目录。

## 8. 细化实施阶段

### P0：资源与边界冻结

产出 `assets/manifest.json`、`docs/PROVENANCE.md`、`docs/THIRD_PARTY_NOTICES.md`、Windows 能力探测表和许可证审计报告。把 96×96 Vitro PNG 标记为 reference 或 approved；来源不清的图标只留内部对照。

### P1：领域核心与持久化

先写 reducer、layout、identity、snapshot、menu、Docklet manifest 和故障恢复的失败测试，再实现纯 Rust 逻辑。完成 schema migration、原子写、backup、reset/import/export，并通过无 Windows 测试。

### P2：Windows 平台闭环

先写 monitor/DPI/AUMID/WinEvent/drop/tray/taskbar adapter contract tests，再实现真实 Win32/COM。完成应用发现、窗口快照、激活、图标提取、Explorer Drop、Recycle Bin 和能力探测。

### P3：原生 Dock surface

先用共享几何通过 geometry/hit-test golden tests，再创建 per-monitor borderless window、D2D/DirectWrite/WIC renderer 和 DComp composition。完成 UIA 元素树、键盘导航和 taskbar coexistence。

### P4：Dock Shell 和自动隐藏

先实现 R1–R12 的 reducer 测试，再接入鼠标、edge/corner/swipe/hotspot、双击、菜单、拖拽重排、子 Dock、launch feedback 和 cancellable auto-hide。

### P5：Launchpad、Shelf、窗口预览和系统面

先完成 Start Menu 搜索、分页、stack fan/grid/list、window preview 和 tray snapshot 的纯模型，再接入真实平台数据。每个异步动作必须有 loading/success/error 状态。

### P6：Docklet 生态

先拒绝未知权限、越界刷新、错误 schema 和崩溃 Docklet，再实现签名/来源、安装/启停/卸载、helper/QuickJS 隔离、CPU/内存预算和官方 Docklet。首发官方 Docklet 为 clock、CPU、RAM、battery、weather provider placeholder、tasks 和 tray。

### P7：材质、动效和美术重构

先通过 token/effect conflict/reduced-motion tests，再实现四层玻璃、噪点、反射、badge、running dot、11 槽效果和两套主题。用自绘向量替代未经审计的旧 PNG；输出 100/125/150% DPI 资产。

### P8：性能、视觉 QA 和发布

完成 screenshot/PrintWindow/UIA/ETW 矩阵、24 小时稳定性、资源泄漏、发布边界扫描、签名和安装包检查。只有所有硬门槛通过，才生成独立 Dock App 安装包。

## 9. 每阶段完成定义

一个阶段只有同时满足以下条件才算完成：

1. 失败测试先出现过，再由最小实现变绿；
2. 真实平台动作和状态快照已闭环；
3. 成功、加载、失败和无障碍路径都存在；
4. 资源来源、许可证和发布边界有记录；
5. 相关性能指标有测量结果；
6. `cargo fmt --check`、相关测试和 `git diff --check` 通过；
7. 阶段提交只包含该阶段范围，不把 Glassboard、Vitro 运行时或临时预览带入新产品。

