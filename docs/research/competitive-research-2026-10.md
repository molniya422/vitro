# 竞品与生态调研报告 — macOS Dock/小组件 & Windows 仿 macOS dock 全景
> 日期:2026-10-24 · 方法:网络调研(官方文档/GitHub/商店/社区/对比评测)
> 用途:Vitro 定位校准与功能借鉴清单

---

## 一、macOS 侧现状

### 1. Dock:macOS Tahoe 26 的 Liquid Glass(2025-09 发布)
- WWDC25 发布 **Liquid Glass** 设计语言,覆盖系统控件/侧栏/工具栏;**Dock 重新设计为半透明液态玻璃、悬浮于桌面**;菜单栏变为**完全透明**
- 含义:本项目已有的 `macos-dock-spec.md`(Sonoma 定量拆解)+ Tahoe 演进方向(更透明、悬浮、连续曲率)= 对标基线;此前社区做的"Liquid Glass CSS 复刻"已转正为官方语言,Vitro 的四层材质实现方向正确,还应加"悬浮感"(更少的基础填充、更强的边缘折射)
- 来源:[Apple 官方新功能 PDF](https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf)、[TechPowerUp](https://www.techpowerup.com/forums/threads/apples-macos-26-tahoe-introduces-floating-transparent-liquid-glass-ui.337895)、[Macworld](https://www.macworld.com/article/2862474/macos-tahoe-prioritizes-productivity-over-liquid-glass.html)

### 2. 桌面小组件(屏幕插件)体系
**原生(Sonoma 起)**:
- 右键桌面 →「编辑小组件」→ 从画廊拖到桌面;小组件间有对齐轮廓吸附
- **iPhone 小组件免 Mac 版直接上桌面**(同 Apple ID via iCloud)——WidgetKit 自动处理渲染差异(iPhone 小组件用 iOS 字体度量,原生 Mac 小组件用 macOS 字号)
- Sequoia 起小组件**自动跟随壁纸着色(tint)**,可在系统设置关闭;支持叠放
- 来源:[WidgetKit 文档](https://developer.apple.com/documentation/widgetkit)、[MacStories Sonoma 评测](https://www.macstories.net/stories)、[Intego 教程](https://www.intego.com/Home/How%20To)

**第三方(macOS)**:
- **Übersicht**:HTML5/CSS/JS 写桌面小组件,可跑系统命令输出,GeekTool 的轻量现代替代;插件即 .widget 文件夹
- **GeekTool**:老牌,四模块(file/image/shell/web),仍可用
- 来源:[sayzlim](https://sayzlim.net/ubersicht-geektool-alternative)、[GeekTool 官网](https://www.tynsoe.org/geektool)
- **对 Vitro 插件系统的印证**:Übersicht 的"HTML/JS 单文件夹小组件"与 Vitro 的"manifest.toml+main.js 记事本可编辑"同构,是被验证过的用户编辑模式

---

## 二、Windows 仿 macOS Dock 项目全景

### 核心竞品对照表

| 项目 | 许可/价格 | 技术栈 | 活跃度 | 关键特性 | 弱点 |
|---|---|---|---|---|---|
| **Seelen UI** | 开源 AGPL-3.0,免费 | **Tauri(Rust+WebView/Svelte)**,18.2k★ | 极活跃(2959 commits,MS Store/winget) | 全桌面环境替换:Weg 工具栏+macOS 式 dock+**平铺窗口管理器(BSP)**+Rofi 式启动器+CSS/JSON 主题+**Widget SDK(Svelte/TS/IPC)**+壁纸取色动态主题+工作区 | WebView 内存高(正是 Vitro 否决的路线);全家桶式上手重;需 WebView2/Edge |
| **MyDockFinder** | Steam 付费买断(低价) | 原生(D3D11 加速,需 Dockmod 组件) | 活跃(我们已抓官网) | MyDock+MyFinder 整套:Dock+顶部菜单栏+**接管 Explorer**;窗口实时预览(含 UWP);三种最小化动画;图标任务进度条;IM 未读角标;WinUI/Mica 模糊可调;SteamCloud 同步 | 纯观感向组织弱;Windows 更新偶发破坏其 shell 改动 |
| **Winstep Nexus**(逆向对象) | 免费+Ultimate 付费 | VB6,~54MB | 活跃 25+ 年 | 最可定制:多 dock/子码头/Shelf/反射/水波;R1-R12 全规格见逆向文档 | 设置如驾驶舱;老架构内存大(GetCursorPos 轮询) |
| **BitDock 比特工具栏** | 免费+插件付费(买断 ~¥69 起) | — | 活跃(bitdock.cn/MS Store) | Mac 风格 dock 任意边缘;**50+ 桌面插件**(日历时钟/天气/截图/取色);可接管托盘 | — |
| **CoolDesk** | 免费 | — | 新品 | **项目感知工作区 dock**(per-project 应用/文件/链接,git 分支/未提交数/开发服务器状态角标) | 无动效;不替换任务栏 |
| RocketDock | 免费 | — | **死亡**(2007 停更) | 经典 | 高 DPI 差 |
| ObjectDock | 付费(Stardock) | — | 弃养 | — | 评测建议"改用 Nexus" |
| XWindows Dock | 免费 | — | 死亡 | — | — |

来源:[cool-desk 对比文](https://cool-desk.com/blog/best-dock-apps-windows)、[Seelen-UI 仓库](https://github.com/eythaann/seelen-ui)、[winstep.net](https://www.winstep.net/nexus.asp)、[bitdock.cn](https://bitdock.cn)、[mydockfinder.com](https://www.mydockfinder.com)、[Reddit r/desktops](https://www.reddit.com/r/desktops/comments/1g2tj8b/any_rocketdock_alternative_modern_opensrouce)

### 品类判断(来自对比文)
- 市场分两支:**眼糖 dock**(仿 macOS 观感)与**工作区 dock**(按当前项目组织内容);公认空白 = "经典 dock 不知道哪些应用属于哪个项目"
- 运行建议:一个 dock + 至多一个小组件工具,堆叠 shell 修改是卡顿根源

---

## 三、Windows 桌面小组件生态(Vitro docklet 的参照系)

| 工具 | 状态 | 说明 |
|---|---|---|
| **Rainmeter** | 2026 仍活跃(论坛/皮肤/GitHub) | 事实标准;皮肤生态仍在出 Material You 自适应类;深定制首选 |
| **GadgetPack**(原 8GadgetPack) | 活跃 | 恢复经典 Windows 小工具,兼容 Win11 |
| **BeWidgets** | 活跃(MS Store 2026-03 更新) | 现代 Win11 风格:时钟/天气/照片/财务/系统监视,设计器式配置 |
| Widget Launcher / Live Tiles Anywhere / Better Widgets | 活跃 | 补齐微软不给的桌面小组件([Pocket-lint](https://www.pocket-lint.com/apps-to-bring-widgets-to-windows-11-desktop)、[XDA](https://www.xda-developers.com/apps-give-desktop-widgets-microsoft-wont)) |
| Themia | 新兴 | Tauri 原生,类 Rainmeter 桌面 live widgets |
| Win11 原生小组件板 | 系统内置 | 触达有限,非桌面自由摆放 |

来源:[rainmeter.net](https://www.rainmeter.net)、[gadgetpack.net](https://gadgetpack.net)、[bewidgets.com](https://bewidgets.com)、[Beebom 皮肤合集](https://beebom.com/rainmeter-skins)、[rainmeter-skin topic](https://github.com/topics/rainmeter-skin)

---

## 四、任务栏改造相邻项目(不进赛道但定风险边界)

- **StartAllBack**(付费 ~$5):经典任务栏/开始菜单恢复,适配 Windows 更新最快,口碑最稳
- **Start11**(Stardock 付费):商业打磨,任务栏工具栏等附加
- **ExplorerPatcher**(免费开源):仍活跃,但**每月累积更新经常打回原形**(2026-05 KB5089573 事件),26H2 insider 上 explorer.exe 崩溃报告多
- **TranslucentTB**:任务栏透明/模糊;与其他 shell 替换冲突
- **Windhawk**:模块化 mod 平台,部分用户为稳定性迁移过去
- **教训(直接支撑 Vitro 架构决策)**:hook/替换 explorer.exe = 每月兼容性地狱;Vitro 的"零 hook、自管分层窗口、不动系统任务栏"路线天然免疫此类风险
- 来源:[ExplorerPatcher issue](https://github.com/valinet/ExplorerPatcher/issues/4801)、[explorerpatcher.net](https://explorerpatcher.net)、[Reddit 对比](https://www.reddit.com/r/Windows11/comments/1ajv9rz/best_taskbar_modification_start11_startallisback)

---

## 五、对 Vitro 的战略结论

### 1. 定位空档被调研确认
"**原生渲染(DComp/D2D)+ 轻量(≤30MB,空闲 0% CPU)+ 现代动效(transitions.dev/macos-parity 双档案)+ 记事本可编辑插件(QuickJS)**"的单体 dock 目前无人占据:
- Seelen UI 占了"开源+全功能",但代价是 WebView 全家桶(内存/启动)——它验证了需求,把"性能敏感、只要一个漂亮 dock"的用户留给了 Vitro
- MyDockFinder 验证了付费意愿(Steam 买断),Nexus 验证了 25 年长青,BitDock 验证了"插件付费"模式
- CoolDesk 指出的空白(项目感知)可作为 Vitro 远期特性(插件可先做)

### 2. 必须遵守的边界
- **Seelen UI 是 AGPL-3.0:一行代码都不能看/搬**;行为与功能观察无妨。Glassboard(MIT)与自研继续作为代码来源
- 不 hook explorer、不替换任务栏(ExplorerPatcher 教训);"MyDockFinder 式接管 Explorer"明确不做

### 3. 功能借鉴清单(按优先级)
1. **Tahoe 悬浮感**:材质四层再减基础填充、增强边缘折射 → 主题引擎"悬浮"预设
2. **MyDockFinder 窗口实时预览 + 图标任务进度条**(已在 M2 清单,优先级上调——两款竞品都重仓此功能)
3. **Seelen 壁纸取色动态主题**(accent 从壁纸提取)→ 主题引擎 M5 加分项,实现成本低
4. **BitDock 插件目录**:50+ 插件的目录化呈现 → Vitro 插件系统加"内置插件商店页"(本地目录,非联网)
5. **叠放小组件**(macOS stacking)+ **iPhone 小组件借位思路** → docklet 可叠放成堆(Stack),点击展开
6. **Übersicht 式 .widget 文件夹分发** → 插件安装 = 解压一个文件夹,双击 .vitroplugin 关联安装

### 4. 分发与生态
- MS Store + winget 双渠道(Seelen 验证);GitHub Releases 签名直发(Glassboard 更新器图纸)
- 插件生态冷启动:官方首发 3 个(CPU/剪贴板/天气)→ 参考 BitDock 的高频件补:日历时钟、截图、取色、网速

---

## 附:全部来源
- macOS Tahoe:[Apple PDF](https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf) · [TechPowerUp](https://www.techpowerup.com/forums/threads/apples-macos-26-tahoe-introduces-floating-transparent-liquid-glass-ui.337895) · [Macworld](https://www.macworld.com/article/2862474/macos-tahoe-prioritizes-productivity-over-liquid-glass.html)
- macOS 小组件:[WidgetKit](https://developer.apple.com/documentation/widgetkit) · [MacStories](https://www.macstories.net/stories) · [Intego](https://www.intego.com/Home/How%20To) · [Fast Company](https://www.fastcompany.com/Tech) · [Übersicht](https://sayzlim.net/ubersicht-geektool-alternative) · [GeekTool](https://www.tynsoe.org/geektool)
- Windows dock:[cool-desk 对比](https://cool-desk.com/blog/best-dock-apps-windows) · [Seelen-UI](https://github.com/eythaann/seelen-ui) · [Seelen MS Store](https://apps.microsoft.com/detail/9p67c2d4t9fb) · [Winstep](https://www.winstep.net/nexus.asp) · [MyDockFinder](https://www.mydockfinder.com) · [Steam 页](https://store.steampowered.com/app/1787090/MyDockFinder) · [BitDock](https://bitdock.cn) · [Reddit](https://www.reddit.com/r/desktops/comments/1g2tj8b/any_rocketdock_alternative_modern_opensrouce) · [macos-dock-for-windows 指南](https://github.com/ronilaukkarinen/macos-dock-for-windows)
- Windows 小组件:[Rainmeter](https://www.rainmeter.net) · [BeWidgets](https://bewidgets.com) · [GadgetPack](https://gadgetpack.net) · [Pocket-lint](https://www.pocket-lint.com/apps-to-bring-widgets-to-windows-11-desktop) · [XDA](https://www.xda-developers.com/apps-give-desktop-widgets-microsoft-wont) · [Beebom](https://beebom.com/rainmeter-skins) · [rainmeter-skin](https://github.com/topics/rainmeter-skin)
- 任务栏改造:[ExplorerPatcher](https://github.com/valinet/ExplorerPatcher/issues/4801) · [explorerpatcher.net](https://explorerpatcher.net) · [Reddit 对比](https://www.reddit.com/r/Windows11/comments/1ajv9rz/best_taskbar_modification_start11_startallisback)
