# 设计语言调研:macOS Dock × Rainmeter(雨滴)皮肤
> 2026-10-24 · 主题:两套"桌面级 UI"的设计语言剖析 —— 一个是闭源标杆,一个是开源生态
> 姊妹篇:macos-dock-spec.md(定量参数)· frontend-design-research.md(MyDockFinder/BitDock)

---

## 上篇:macOS Dock 的设计语言

### 1. 基因谱系(25 年四代演进)
| 代 | 年份 | 材质语言 | Dock 形态 |
|---|---|---|---|
| **Aqua** | 2000(Panther~Mavericks) | "Lickable"水感光泽:半透明条纹、水滴按钮、**果冻质感 Dock 槽** | 白色浅槽承载珍珠般图标;Genie 效果首秀(2000-01-05 Jobs 演示) |
| 中间态 | 2014(Yosemite)~2020(Big Sur) | 扁平化+轻微毛玻璃;Big Sur 起 **Dock 变悬浮圆角玻璃条**(去槽),图标改 squircle 写实风 | 半透明模糊背板,放大仍在 |
| **Sonoma 量化期** | 2023 | 四层液态玻璃定型(模糊20/22+饱和、18%/25% 填充、噪点、1px 内高光/内阴影) | 本项目 macos-dock-spec.md 全参数 |
| **Liquid Glass** | 2025(WWDC,Tahoe 26) | 官方语言:"**光学特性+流动感**";控件/侧栏/菜单栏全量玻璃化;**菜单栏完全透明**;Dock 悬浮并**折射背后桌面**;首次全平台统一(iOS/iPadOS/macOS/visionOS) | Dock = 一块会折射桌面的浮起玻璃 |

> 关键连续性:25 年里 Dock 的三大交互签名从未变过——**放大(抛物线联动)、Genie/缩放最小化、运行指示点**。设计语言换了四代,交互骨架一脉相承。这就是"设计语言 vs 交互骨架"的分层:材质随时代,骨架随习惯。

### 2. 支撑 Dock 的三大 HIG 设计原则(官方)
- **Clarity 清晰**:文本任何尺寸可辨、图标精确、装饰适度——对应 Dock 的写实习得图标 + 13pt medium 标签
- **Deference 遵从内容**:UI 服务内容不抢戏——对应"无外描边无外投影、层级全靠材质"的克制(区别于廉价毛玻璃的核心)
- **Depth 深度**:视觉层级 + 真实动效转场帮用户建立空间心智——对应 Genie(窗口去哪了永远可追)、放大(物理透镜隐喻)、分层模糊(近实远虚)

### 3. 设计语言要素拆解(Dock 本体)
- **材质**:四层结构模拟真实玻璃(模糊+饱和补偿 / 半透明填充 / 噪点消塑料感 / 1px 内高光+内阴影造"薄片体积");Liquid Glass 世代再加边缘折射。**玻璃不是装饰,是深度线索**。
- **形态**:squircle 连续曲率(图标 20% 边长圆角、Dock 圆角=总高/2 两端半圆);图标底对齐、放大向上生长(位移=增量/2)。
- **图标学**:写实渲染(渐变+高光+投影的"实体物件"感),图标即品牌;系统图标矢量、应用图标 PNG 纹理。
- **动效哲学**:**物理真实感优先于炫技**——放大是透镜/抛物线,启动弹跳是衰减弹簧(12/6pt 两跳),Genie 是质量守恒的形变;无弹性回弹的克制缓动 `cubic-bezier(0.25,0.46,0.45,0.94)`;**还原=逆放**(空间记忆)。
- **状态即信息**:4pt 运行点(深色白60/浅色黑40)、红角标(1px 内高光)、进度条、悬停亮度+5%——全部是"图标本体微变",不加框不加底。
- **空间模型**:屏幕底边的一条"架子"(shelf);四角 bump/热点=把屏幕边缘当作隐形按钮;菜单栏与 Dock 构成"上下框线",中间留给内容(Deference)。

### 4. 可迁移的设计法则(写给 Vitro)
1. 材质表达层级,不用描边表达层级
2. 动效服务空间心智(东西去哪了要可见、可逆)
3. 状态写在图标本体上,不造新容器
4. 交互骨架稳定 25 年不变,材质每 5-8 年换血——架构上把"材质(主题)"与"交互(行为规格)"分离(我们已是这么分的:R 规格与 macos-parity 材质表)

---

## 下篇:Rainmeter(雨滴)皮肤的前端设计

### 1. 技术构造如何塑造设计语言(先讲约束)
皮肤 = INI 文本(measures 数据源 + meters 视图件)+ 可选 Lua;Shape 矢量件、Roundline 弧形件驱动百分比;透明度靠颜色 alpha 通道,**无系统级实时模糊**(玻璃感全靠半透明 PNG/渐变伪造);meter 不能动态复制(Lua 生成 INI 绕行)。
→ 约束产物:**皮肤天然偏"扁平图层+文字排版",做不了真玻璃**;于是生态分化出靠排版取胜的极简派和靠 PNG 拼贴的拟物派。

### 2. 生态的设计光谱(2026 主流,DeviantArt 1000+ 套)
| 流派 | 代表 | 设计特征 |
|---|---|---|
| **极简文字派**(主流) | Mond、LIM!T、Senja、Clouds | 大号细字重时钟 + 大量留白;LIM!T 用"字体内填充进度"表达数据;仿佛直接写在壁纸上(Wisp 的"无边界"理念) |
| **玻璃拟态派** | Frost Glass、Speed | PNG 磨砂+弧形面板;Frost Glass 自带底部 macOS 式 Dock;Speed 仪表盘式 CPU/RAM |
| **科技/HUD 派** | Cyberpunk 2077、Advansity、Everon | 荧光色、等宽字、传感器仪表风;信息密集 |
| **拟物怀旧派** | TRANSFORMEX、Enigma(2009 官方默认) | 3D 图标、JetAudio 时代质感;Enigma 定义了"模块可重排"范式 |
| **整桌面套件派** | MacOS Monterey/Big Sur 套件、Windows 12 概念、Gemini Suite | 直接把整个 OS 视觉搬到桌面;含 dock+顶栏+小组件全家桶 |
| **顶栏套件** | **Droptop Four** | Rainmeter 生态里的"MyFinder":悬浮顶栏+下拉菜单树+搜索+托盘溢出管理+15 个小应用(Gmail/天气通知);22+ 语言;免费+付费支持者版;社区主题 |

### 3. Rainmeter 皮肤的设计词汇表(提炼)
- **布局三原位**:左侧信息栏 / 底部 Dock / 角落小组件;时钟日期≈100% 出现率,系统监控与音乐播放器次之
- **数据可视化三板斧**:Roundline 圆环进度(仪表盘隐喻)、字体填充进度(LIM!T)、可视化器条(音乐)
- **"写在壁纸上"美学**(Wisp/Mond 代表):无面板、无边框、文字+细线直接叠壁纸——与 macOS"材质承载层级"正好相反的另一种 Deference(用壁纸当背景板)
- **社区驱动**:DeviantArt 分发、变量化配色(用户改几个变量就换肤)、Droptop 的社区主题/应用市场

### 4. 对 Vitro 的启示
1. **Droptop Four = 需求验证**:Rainmeter 生态里长出一个"顶栏+下拉菜单+启动器"套件且能收费——证明"dock/顶栏 + 可编程小组件"组合有付费用户;我们 QuickJS 插件 + BitBar 式顶栏(远期)方向正确
2. **极简文字派是最普及的设计语言**:Vitro 主题引擎应内置一个"裸排版"主题(无玻璃、纯字重排版)——低成本覆盖最大审美人群
3. **PNG 玻璃 vs 真玻璃是我们的代差优势**:Rainmeter 做不了实时模糊,Webley/D2D 合成器免费做——营销点
4. **变量化换肤**:Rainmeter 用户改 3 个变量就换色——对应主题 TOML 的热重载 + 复刻配置码,已对齐
5. **负面教训**:信息密集 HUD 风是审美小众;整桌面套件(MacOS Monterey 套件)维护成本高、假 dock 交互残缺——Vitro 不做"贴图式整桌面",只做真 dock

## 附:来源
macOS:[HIG 设计原则](https://developer.apple.com/design/human-interface-guidelines/design-principles) · [HIG 主页](https://developer.apple.com/design/human-interface-guidelines) · [Aqua 演进(Wikipedia)](https://en.wikipedia.org/wiki/Aqua_(user_interface)) · [uBar Dock 史](https://ubarapp.com/dock-history) · [UX Planet Aqua 遗产](https://uxplanet.org/apple-aqua-exploring-the-legacy-of-macos-x-user-interface-3a11eb9b7dba) · [Apple Newsroom Liquid Glass](https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design)
Rainmeter:[官网](https://www.rainmeter.net) · [Beebom 2026 25 皮肤](https://beebom.com/rainmeter-skins) · [Droptop Four 官网](https://www.droptopfour.com) · [皮肤加载机制(论坛)](https://forum.rainmeter.net/viewtopic.php?t=44744) · [Shape meter 文档](https://docs.rainmeter.net/manual/meters/shape) · [DeviantArt 皮肤库](https://www.deviantart.com/rainmeter/gallery/23941137/skins) · [Mond](https://www.deviantart.com/apexxx-sensei/art/Mond-762455575)
