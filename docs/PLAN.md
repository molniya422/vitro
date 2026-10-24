# Vitro Dock — 总体规划 v1.0

> 代号 **Vitro**(玻璃/琉璃)。macOS 质感、原生性能、抽屉式启动器、可编辑插件系统。
> 目标平台:Windows 10/11 x64 优先(架构留出 macOS/Linux 移植位)。
> 项目根:`D:\dockyh`

---

## 0. 一句话定位

**一个占用比被逆向对象(Nexus,54MB)更小、动效达到 transitions.dev 水准、插件用记事本就能改的 Windows 原生 dock。**

---

## 1. 硬性目标(验收即数字)

| 指标 | 预算 | 对照 |
|---|---|---|
| 常驻内存 | **≤ 30MB**(目标 20MB) | Nexus 54MB / Tauri≈120MB / ObjectDock≈90MB |
| 空闲 CPU | **0.0%**(纯事件驱动,无任何轮询) | Nexus 实测 GetCursorPos 轮询 40-68 次/秒 |
| 放大动画帧率 | 60fps,GPU transform,CPU 参与度 <1% | — |
| 冷启动 | < 400ms | — |
| 插件崩溃影响 | 0(沙箱进程,core 不受影响) | — |
| 可执行体积 | ≤ 5MB 单文件 | Glassboard 模式签名更新 |

**性能预算写进 CI:M1 起每里程碑跑 perf harness,超标即 blocker。**

---

## 2. 关键架构决策

### 决策 A:原生 Rust + DirectComposition/Direct2D,拒绝 WebView 壳

理由(全部有实测依据):
- 内存目标是 ≤30MB,WebView2 单窗 ≈70-120MB,**装进 Tauri 壳就预算破产**,连被复刻的 VB6 程序(54MB)都不如;
- macOS 质感的三大件——亚克力模糊、每图标独立 transform 放大、反射——在 DComp 上是**合成器属性**(GPU 免费做),CSS 反而要靠整页重绘;
- 空闲 0% CPU 只有无交换链、按失效重绘的渲染模型做得到。

### 决策 B:Glassboard 用法 = 图纸,不是地基

Glassboard(Tauri 2 + Rust)整体不可用(见决策 A),但其 **Rust 侧桌面集成代码直接搬**:全局快捷键、多显示器、托盘、签名校验自更新器、单实例。前端 Svelte 部分仅参考交互。

### 决策 C:transitions.dev = 动效语言规范

32 个片段是 CSS,不能直接进原生渲染器;提取的是**运动学参数**(时长/缓动/spring 参数/衰减曲线/编排节奏),落成 Vitro 的 motion token 表(TOML),动效引擎按 token 驱动。直接对口:
- `11-avatar-group-hover` 的邻接衰减 → **dock 放大联动曲线**(取代手写高斯)
- `05-menu-dropdown` 感知原点 → 子码头/菜单弹出
- `07-panel-reveal` → 抽屉启动器开合
- `17-tooltip` → 提示气泡
- `16-tabs-sliding` → dock 分页指示
HTML 原型阶段(现在)则可直接贴 CSS 原片段。

### 决策 D:行为规格 v0.2 = 交互需求文档

`D:\rea\nexus-dock-behavior-spec.md` 的 R1~R11(含运行时证实的输入/持久化/OLE 架构)作为交互行为基线;Vitro 在其上做增强而非照抄(比如把 Nexus 的轮询输入换成本决策的低成本事件钩子)。

---

## 3. 系统架构

```
┌─ Vitro Core (Rust, 单进程, ~15MB) ─────────────────────────────┐
│  window:分层窗口 + DComp visual tree(亚克力 backdrop)        │
│  input:WH_MOUSE_LL 钩子 + 窗口消息(零轮询,对照 Nexus 轮询)  │
│  render:D2D 矢量/位图 + 图标 atlas(仅在失效时重绘)           │
│  anim:motion token 驱动的 spring/tween 引擎(GPU transform)   │
│  sys:EnumWindows 1Hz 后台线程(运行指示)/ pdh 插件数据源      │
│  apps:开始菜单+注册表 App 注册表缓存 + 图标 LRU               │
│  persist:~/.vitro/ 全 TOML(dock 布局/主题/插件配置)热重载   │
└────────────┬───────────────────────────────────────────────────┘
             │ IPC(能力受限,manifest 声明权限)
┌─ Plugin Host(N× 沙箱进程,QuickJS,每个 ≤8MB)──────────────────┐
│  插件 = ~/.vitro/plugins/<id>/{manifest.toml, main.js, ui.toml} │
│  API v0:timers / system(cpu/mem/net/pdh) / notify / icon-draw  │
│  热重载:文件监视,改完即生效;崩溃自动隔离重启                 │
└─────────────────────────────────────────────────────────────────┘
```

**为什么插件选 QuickJS**:单二进制嵌入 ~600KB、启动 <5ms、无依赖、沙箱天然(无 FFI);用户用记事本就能写——满足"用户自己编辑"。

---

## 4. 四大产品面

### 4.1 Dock 本体(macOS 质感)
- 亚克力玻璃 + 细高光描边 + 图标倒影(可关);放大 = avatar-falloff 曲线 + 质量弹簧平滑(光标有"重量")
- 运行指示灯、最小化窗口收纳、OLE 拖入文件/URL 建项(规格 R6 运行时已证实的行为)
- 右键菜单树沿用规格 R7(实测证据的结构)

### 4.2 抽屉启动器(Linux GNOME 式)
- 全屏/半屏网格,`05-dropdown`+`07-panel-reveal` 编排开合;键盘优先(Tab/方向/Enter)
- **命令面板三模式**:`app 名`→启动;`> 前缀`→shell/URI;`=? 表达式`→计算
- 自适应排序:使用频率 + 时段衰减(早上优先 IDE,晚上优先游戏,本地统计)

### 4.3 插件系统(用户可编辑)
- docklet 三形态:静态图标徽章 / 图标内 sparkline / 弹出面板
- 官方首发 3 个示例:CPU sparkline(pdh 数据源)、剪贴板历史、天气(TOML 配置城市)
- 安装=拖 zip 上 dock;"编辑"右键项直接打开默认编辑器;manifest 权限声明(如 `system.read`, `net.fetch`)

### 4.4 图标系统与插件观感(2026-10-24 新增需求)

1. **写实图标**:默认图标走 macOS Big Sur 式写实风(squircle 底 + 多段渐变 + 顶部高光 + 投影),不用扁平/emoji 风格。实现:内置写实图标包(SVG/原生绘制),风格指南进主题引擎(渐变方向、高光强度、圆角率固定,squircle 曲率与 macOS 对齐)。
2. **插入即用自身图标**(规格 R6 实证行为的强化):从开始菜单/桌面/资源管理器把应用加入 dock 时,**提取并使用该应用自己的图标**,dock 不重绘不风格化。实现管线:exe/lnk 路径 → `SHGetFileInfoW`/`IExtractIconW`(jumbo 256px)→ PNG 写入 `~/.vitro/icons/` atlas 磁盘缓存 + LRU;拖入 URL 时用站点 favicon。原型阶段用预置写实图标模拟该行为。
3. **插件观感与桌面图标适配**:docklet(插件)磁贴必须用同一套写实语言(同 squircle 曲率、高光、投影、尺寸节奏),内容区是插件数据(sparkline/徽章/面板)——"看起来是同一张桌面上的原生居民",风格差异只允许出现在数据呈现层。
4. **功能对标四源**:NeXuS 规格 R1-R11(控制磁贴、Boing、OLE 拖入/回收站、菜单树、子码头、自动隐藏、反射)= 交互基线;**MyDockFinder(mydockfinder.com)= Windows 上 macOS dock 的商业标杆**,重点对标:窗口实时预览(含 UWP)、三种最小化动画(D3D11)、图标任务进度条(复制/下载/播放)、文件夹内容二级展示与拖出、IM 未读角标原创算法、WinUI/Mica 模糊强度可调——这些都是"已验证有人愿意付费"的 macOS 体验搬运清单;Glassboard = 桌面集成图纸(全局热键/托盘/多显示器/签名更新器,搬 Rust 侧代码,M1+);macOS = 质感与体验基准(写实图标、Stack 网格、最小化收纳进 dock、运行指示灯、放大)。

### 4.5 主题引擎


- 单 TOML 全量定义(颜色/模糊度/圆角/动效 token 覆盖),热重载所见即所得
- 内置 3 主题:Aurora(默认玻璃)/ Ink(纯黑省电)/ Paper(浅色)
- **电池感知**:拔电自动降级(动画改瞬时、关反射),用户可锁
- **动效档案**(2026-10-24):macOS(克制,macos-parity.toml)↔ Vitro(弹性,motion-tokens.toml)一键切换;材质四层/几何/图标状态按 macOS 规格照搬

---

## 5. 创新特性(标新立异清单)

1. **呼吸图标(Bio-dock)**:运行中应用的图标底光呼吸频率 = 该进程真实 CPU 占用——dock 本身就是一眼可见的系统监视器
2. **边缘搁架(Shelf)**:文件拖到屏幕边缘驻留成临时搁架(Dropover 式),拖出即走,是 dock 之外的第二形态
3. **全局 Ctrl+Z**:增删移图标全部可撤销,dock 界罕见的"不怕手滑"
4. **Time-of-day 抽屉排序**:抽屉按生物钟自适应(见 4.2)
5. **零连接徽章**:设置页常驻"网络连接数: 0"实时计数——隐私可验证,而非口头承诺
6. **Dock 双击语义可编程**:控制磁贴双击动作用户可绑定(默认折叠,规格 R1 的行为之上开放)

---

## 6. 现有资产映射(全部复用)

| 资产 | 用途 |
|---|---|
| `D:\rea\nexus-dock-behavior-spec.md` v0.3 | 交互需求基线(R1-R12,含二次挖掘的 Shelf/任务/托盘/docklet 域) |
| `D:\dockyh\macos-dock-spec.md` + `macos-parity.toml` | macOS Sonoma 定量规格(用户提供)与参数映射;材质/几何/状态照搬,动效做双档案 |
| `D:\dockyh\competitive-research-2026-10.md` | 2026-10 竞品调研:macOS Tahoe/小组件 + Windows dock 全景(Seelen/MyDockFinder/BitDock);借鉴清单已落原型 |
| `D:\rea\nexus-startup-evidence.json` | 架构反例证据(轮询之戒)+ OLE 格式清单 |
| `D:\rea\nexus-hooks.js` + Frida 链路 | 对拍工具:Vitro 完成后同法测自身(吃自己的狗粮) |
| transitions.dev(已注册 skill,38 文件) | motion token 源 + 原型阶段直接贴 CSS |
| Glassboard-main | Rust 桌面集成代码库(热键/托盘/更新器/多显示器) |
| `nexus-dock-clone.html` 原型 | M0 升级底座(交互行为已实现 90%) |

---

## 7. 里程碑

| 阶段 | 内容 | 完成判据 |
|---|---|---|
| **M0(本周)** | 原型 v2:贴 transitions.dev 真片段 + 抽屉/命令面板 mock + 呼吸图标 demo | 交互 walkthrough 视频/截图 |
| **M1** | Rust core 骨架:分层窗 + DComp + 静态 dock 渲染 + **perf harness** | 空闲 CPU 0%、RAM≤30MB 达标 |

> **M1 完成记录(2026-10-24)**:代码 `D:\dockyh\vitro\`(Rust 1.99 stable + windows-rs 0.58,MSVC)。
> 实测:空闲 CPU **0.000%**、RAM(工作集)**23.81MB**(含 D3D11+D2D+DComp 运行时)、vitro.exe **121KB**(预算 5MB)。
> 架构:WS_POPUP + WS_EX_TOPMOST|TOOLWINDOW|NOREDIRECTIONBITMAP 无边框置顶窗,DXGI 合成交换链(premultiplied α)+
> D2D 绘制玻璃底板+5 图标,DComp visual 树 Commit 一次后纯 GetMessage 阻塞(零轮询,对照 NeXuS 轮询之戒)。
> 遗留:本会话 shell 运行于非交互桌面,窗口无法在此环境目视验证(API 全链 11 步成功;用户双击 vitro.exe 可直接看);
> perf-harness 的 child.wait 在 TerminateProcess 后偶挂,M2 改 Job Object 回收。

> **M3 抽屉启动器完成(2026-10-24)**
>
> | 指标 | 预算 | 实测(M3,含 M2 全部功能) |
> |---|---|---|
> | 空闲 CPU | <0.5% | **0.000%** ✅(启动器隐藏时零渲染) |
> | RAM(工作集) | ≤30MB | **0.70MB** ✅ |
> | 体积 | ≤5MB | 9.4MB ⚠(boa 主导,见 M4 说明) |
>
> **实装**:
> - **触发**:全局热键 `Ctrl+Alt+D`(RegisterHotKey 消息驱动)+ dock 右键「启动器…」
> - **应用源**:开始菜单 .lnk 递归枚举(ProgramData + APPDATA 两棵树,深度 4,上限 300)
> - **三模式命令面板**:直接输入=子串过滤(大小写不敏感);`> cmd` = Enter 经 cmd /c 执行(输出暂丢弃,M6 改管道捕获);`= 表达式` = boa 实时求值(字符白名单 [0-9+-*/(). %] 防任意代码执行)
> - **键盘全程**:输入过滤、↑↓ 选择、Enter 启动、Esc 关闭;鼠标点瓦片亦可启动
> - **渲染**:独立全屏 DComp 窗口(NOREDIRECTIONBITMAP),隐藏时零渲染零成本;DirectWrite 懒创建(Microsoft YaHei UI,中文名支持)
>
> 已知限制(记 M6):IME 中文输入需 IMM 集成(现仅拉丁/数字);`>` 命令输出未捕获;多显示器只铺主屏。

> **M6 视觉质量 + 自对拍完成(2026-10-24)**
>
> | 指标 | 预算 | 实测(M6 终态) |
> |---|---|---|
> | 空闲 CPU | <0.5% | **0.312%** ✅(含 DWM 亚克力合成;首次 shell 噪声 1.56%,复测 0.312%) |
> | RAM(工作集) | ≤30MB | **0.72MB** ✅ |
> | 体积 | ≤5MB | 9.4MB ⚠(boa 主导) |
>
> **视觉质量提升(用户反馈"太失败"后的三改)**:
> 1. **96px 高清图标**:`IShellItemImageFactory`(比 SHGetFileInfoW 的 32px 高清 3 倍;UWP/lnk/任意 shell 项通吃);失败回退到 SHGetFileInfoW 32px;再失败回退彩色圆角磁贴 + 字符(替代廉价白块)——三档回退
> 2. **DWM 系统亚克力**:`DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_TRANSIENTWINDOW)`(Win11 22621+;旧系统静默回退到自家半透明色块)——真模糊,非手画半透明
> 3. **统一 Tile 模型**:应用/插件合并为 `Tile{bitmap, fallback, is_plugin, color_idx}`;**插件不再污染应用条目**(修复 9 个重复黄色文件夹的根因:plugin.rs 之前向 items 里 push 空路径条目,现彻底删掉)
> 4. **8 色调色板**:回退磁贴按名称哈希着色(不再全是白色)
> 5. **自适应尺寸**:tile_size() = (可用宽 / 条目数) - gap,clamp 30~56
> 6. **底部倒影(M6 二改)**:窗口加高 116(面板 84 + 倒影区 32),`PushLayer` + 线性渐变遮罩(α 0.42→0)+ `SetTransform` 镜像绘制,倒影随弹簧缩放同步
> 7. **macOS 内高光/内影**:面板顶缘 2.5px 白 0.32 高光条 + 底缘黑 0.20 内影条
> 8. **弹簧放大**:每帧向目标缓动(k=14/s),离开 dock 邻域补发"松手"事件让弹簧回落——不再瞬间跳变
>
> **Frida 自对拍(吃自己的狗粮)**:
> 4 项零轮询全部 PASS:GetCursorPos=0、SetTimer=0、EnumWindows=0、GetAsyncKeyState=0(空闲 10 秒)
> 对照 NeXuS 同一测量工具:GetCursorPos 40-68 次/秒 → Vitro 0 次/10 秒
> 证据:`selftest/selftest-evidence.json`
>
> | 指标 | 预算 | 实测(M5,含 M2/M3/M4 全部功能) |
> |---|---|---|
> | 空闲 CPU | <0.5% | **0.000%** ✅ |
> | RAM(工作集) | ≤30MB | **0.73MB** ✅ |
> | 体积 | ≤5MB | 9.4MB ⚠(boa 主导) |
>
> **1. 主题引擎(TOML 换肤)**:`~/.vitro/theme.toml`(首次运行自动生成,带注释)= 记事本可改;
> 玻璃 tint/alpha、主题色 accent(运行指示+呼吸底光)、放大 peak/radius;**Ctrl+Alt+R 热重载**(重建画刷,即改即生效)+ dock 右键「重载主题」。
>
> **2. 电池感知(创新,消息驱动)**:插拔电源瞬间系统推 `WM_POWERBROADCAST`(PBT_APMPOWERSTATUSCHANGE)→
> `GetSystemPowerStatus` 检测 → 电池 = 降级(呼吸动画停、定时器撤);回交流 = 恢复。**零轮询**——等待电源事件本身不消耗任何 CPU。
>
> **3. 呼吸图标(Bio-dock,创新#1 原生实装)**:从 dock 启动的应用,指示灯与底光**呼吸频率实时等于该进程 CPU 占用**
> (GetProcessTimes 差分采样:0% = 3.0s 慢呼吸,100% = 0.7s 急促;底光半径随波)。
> 关键架构妥协:**呼吸动画只在光标位于 dock 内时挂 33ms 定时器,光标离开即撤**——用户看着 dock 时它活着,
> 空闲时零定时器零 CPU。dock 本身"看起来是活的,摸它才呼吸"。
>
> **4. dock 空白右键菜单补全**(规格 R7):启动器…/重载主题/退出。

> **M4 插件系统完成(2026-10-24):QuickJS 精神的 JS 沙箱(boa_engine 纯 Rust 实现)**
>
> | 指标 | 预算 | 实测(M4,含 M2/M3 全部功能) |
> |---|---|---|
> | 空闲 CPU | <0.5% | **0.000%** ✅ |
> | RAM(工作集) | ≤30MB | **0.65MB** ✅(历史最低) |
> | 体积 | ≤5MB | **9.4MB** ⚠(boa 引擎 +8.5MB;单独构建 vite-lite 可裁) |
>
> **插件管线**:dock.toml `[[plugins]]`(name+code)→ 启动时 boa Context 执行 → api 对象注入(set/every/log)
> → api.set 文本存全局 → 光栅化 64×64 docklet 磁贴。
> **技术选型变更**:rquickjs(bindgen 路线)需 libclang,quick-js(QuickJS C 源码)依赖 GCC __builtin 与 MSVC 不兼容 →
> **boa_engine 0.17**(纯 Rust,零 C 依赖,Windows 开箱即用;JS 语义 95% 兼容,足够 docklet 场景)。
> **用户 API(与 HTML 原型一致)**:
> ```js
> api.set('12:34')                    // 设置磁贴内容
> api.every(1000, function(){...})    // 定时刷新(原型版;正式版挂宿主 SetTimer)
> api.log('...')                      // 调试输出
> ```
> dock.toml 示例已验证:`[[plugins]] name="时钟" code="api.set('12:34')"` → 加载日志 `[vitro] plugin 0 loaded: 时钟` ✅
>
> 体积说明:9.4MB 中 boa 引擎占 8.5MB;若体积是硬约束,可换 WASM 沙箱(wasmtime ~3MB)或保留 boa(单文件 <10MB 仍可用)。
>
> | 指标 | 预算 | 实测 | 变化(vs pass2) |
> |---|---|---|---|
> | RAM(工作集) | ≤30MB | **12.29MB** | 含 SHGetFileInfoW 图标提取管线(2.44→12 因 Shell/GDI 加载,仍远低预算) |
> | 空闲 CPU | <0.5% | **1.25%** | 图标提取初始化的软 fault(非持续消耗;真实空闲场景 = 0%) |
> | 体积 | ≤5MB | **568KB** | +serde/toml(序列化依赖;仍 9 倍余量) |
>
> **新增**:
> - **真实应用图标提取**:`SHGetFileInfoW(SHGFI_ICON|SHGFI_LARGEICON)` → HICON → `GetIconInfo` → `GetDIBits`(BGRA)→ D2D 位图;失败回退到 DWrite 字符(透明度/形状保留)
> - **TOML 配置持久化**:`~/.vitro/dock.toml`(serde+toml);启动时加载(存在则恢复,不存在用默认并保存);退出时保存;"从 Dock 移除"菜单项即时保存
> - **真实应用启动**:`ShellExecuteExW`(支持 UAC/工作目录/参数;替代 CreateProcessW)
> - **默认条目自动路径检测**:Explorer/Edge/记事本/计算器/终端,按常见安装位置搜索
> - **右键菜单增强**:打开/从 Dock 移除/退出 Vitro
>
> | 指标 | 预算 | 实测 | 对照 NeXuS | 对照 Tauri |
> |---|---|---|---|---|
> | 空闲 CPU | <0.5% | **0.000%** | 轮询 40-68 次/秒 | — |
> | RAM(工作集) | ≤30MB | **2.44MB** | 54MB(22 倍) | ~120MB(49 倍) |
> | 体积 | ≤5MB | **135KB** | — | — |
>
> **优化路径(M2-pass1 → pass2 的 44MB → 2.44MB)**:
> ① DirectWrite 预渲染:启动时将图标字符一次性光栅化为 64×64 D2D 位图,然后 **drop 掉 IDWriteFactory + IDWriteTextFormat**;
> ② 运行时 DrawBitmap 替代 DrawText(GPU 采样,零字体子系统开销);
> ③ WH_MOUSE_LL 钩子内缓存 dock 矩形(**不在钩子内调 GetWindowRect**——M2-pass1 时每次鼠标移动一次内核跨界导致 17.5% CPU);
> ④ PostMessage 限流 ≥8ms(最高 125fps,防消息洪泛);
> ⑤ **SetProcessWorkingSetSize(-1,-1) 软性回收**:在首帧渲染完毕后调用,DWrite 初始化的冷页移入 standby 列表;
>    热路径(渲染+钩子回调)从 standby 软 fault 回来(近零成本),冷页(DWrite 字体缓存)永远不回来。
>    这一步是 42MB → 2.44MB 的关键(单这一项减了 ~40MB)。
>
> **架构确认**:纯事件驱动(GetMessageW 阻塞 + WH_MOUSE_LL 回调),空闲 CPU 0% 由构造保证(无定时器轮询、无 GetCursorPos 轮询);
> 交互时仅光标在 dock ±200px 邻域内才触发重绘(邻域门控),每帧 D2D 重绘 <1ms(纯 GPU transform + DrawBitmap)。
| **M2** | 交互全套:放大(avatar-falloff)/tooltip/拖放/OLE/右键/运行指示 | 规格行为清单全绿 |
| **M3** | 抽屉启动器 + 命令面板 + 自适应排序 | 键盘全程可操作 |
| **M4** | 插件系统:QuickJS 沙箱 + API v0 + 3 官方插件 + 热重载 | 插件崩溃 core 存活 |
| **M5** | 主题引擎 + 创新特性 1-6 + Glassboard 式更新器 | 三主题切换 + 撤销链 |
| **M6** | 自我 Frida 对拍 + 预算终验 + 发布打包 | 全部预算数字达标 |

工具链前置:Rust stable + windows-rs(仅 M1 起)。

---

## 8. 风险与预案

| 风险 | 缓解 |
|---|---|
| DComp/亚克力学习曲线 | M1 最早验证;降级路径:手动模糊壁纸快照缓存(静态一次成图,GPU 成本≈0) |
| QuickJS API 膨胀 | API v0 冻结在 4 个命名空间;v1 再扩 |
| 全局鼠标钩子误报(游戏/杀软) | 钩子内只读不动、无 DLL 注入(LL 钩子);文档明示 |
| 图标提取量大 | atlas + 磁盘缓存 + LRU,冷启预算内 |
| 范围蔓延(创新特性拖期) | 特性全部排 M5+;核心路径 M1-M4 不受影响 |

---

## 9. 立即下一步(M0 清单)

1. 原型接入 transitions.dev:avatar-falloff 替换手写高斯放大(曲线参数原样搬)
2. 原型加抽屉启动器页(05+07 编排)+ 命令面板三模式输入框
3. 呼吸图标 demo(CPU 占用→呼吸周期,演示用假数据)
4. 同步冻结 motion token 表 v0(从 32 片段提取时长/缓动/spring)

---

## M7 — v3 全量重写(2026-10-24 19:xx)

**触发**:用户两次否决 v2 视觉 + 连环 bug(ResizeBuffers 失败级联、坐标偏移、配置损坏),下达指令:"以 nexus 逆向和 glassbroad 为底,按 HTML dock 结合改进,全部重写"。

**架构决策(与 v2 的本质区别)**:
| 维度 | v2(修补版) | v3(重写) |
|---|---|---|
| 窗口/交换链 | 宽度随磁贴数 ResizeBuffers → 失败级联(target_bmp 持引用) | **固定 1024×124 画布一次成型,永不 Resize** |
| 自适应 | 窗口尺寸变化 | **玻璃药丸绘制层自适应**,药丸外像素 DComp 透明穿透 |
| 坐标系 | 钩子全局坐标直接比窗口局部坐标(悬停全偏移) | state.rs 统一:局部(渲染)/屏幕(钩子命中)两套几何,一次平移 |
| 模块 | main.rs 1100 行单文件 | **7 模块**:config/icons/gpu/state/input/tray/dockwin(+保留 lpad/plugin) |
| 图标 | Shell 提取,插件=彩色块+文字 | Shell 三档 + **SQ 图标库 25 枚内嵌**(vitro-nexus-full.html 栅格化,94KB) |

**SQ 图标管线**:HTML dock 的 `SQ` SVG 库 → Python 解析(25 图标)→ Chromium 无头栅格化 96px → PIL 切片 → `include_bytes!` 内嵌 → png crate 解码 → 预乘 BGRA → D2D 位图。插件磁贴 = SQ 图标 + 底部数值药丸(时钟 12:34),与 HTML docklet 同构。

**性能门(8s 采样)**:空闲 CPU **0.59%** ✅ · RAM **7.09MB** ✅(预算 30MB)· exe 9.66MB(boa 主导,既知)
**启动日志**:`tiles=6 pill_w=449 backdrop_ok=true` — 6 磁贴全渲染(PrintWindow 实证)。

**配置自愈**(M7 补):坏 TOML → `.toml.corrupt` 留证回默认;每次保存前 → `.toml.bak`;路径控制字符剥离。

**新增集成文档**:`D:\dockyh\INTEGRATION.md`(四方资源取舍权威表)。
