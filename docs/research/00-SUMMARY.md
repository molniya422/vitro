# Vitro 设计知识库总纲(RESEARCH SUMMARY)
> 2026-10-24 汇总版 · 综合 docs/research/ 全部 9 份调研 + specs/ 3 份参数表,消除重复、统一数字、给出唯一权威结论。
> 本文档 = 设计与产品的**单一事实来源(SSOT)**;各分篇为证据链,与本文冲突时**以本文为准**并回改分篇。

---

## 0. 调研地图(9+3 份资产各管什么)

| 文档 | 管什么 | 关键产出 |
|---|---|---|
| nexus-dock-behavior-spec.md | **交互行为基线**(逆向实证 R1-R12) | 行为规则+证据链 |
| macos-dock-spec.md | macOS Dock 定量参数(Sonoma) | 材质/动效数值 |
| competitive-research-2026-10.md | 竞品全景与市场空档 | 定位结论 |
| frontend-design-research.md | MyDockFinder×BitDock 前端设计 | 借鉴清单 |
| design-language-research.md | macOS Dock 设计语言 25 年演进 + Rainmeter 光谱 | 法则提炼 |
| plugin-design-language.md | 插件三层设计规格(风格/思路/实现) | 逐插件规格表 |
| tech-stack-research.md | 技术栈(库/语言/实现方式) | 三层选型结论 |
| skillry-premium-design-research.md | 高级感方法论(品味即产品) | 反 AI 味清单 |
| design-skills-mcp.md | 设计类 Skills/MCP 生态 | 验证闭环结论 |
| specs/motion-tokens.toml | Vitro 弹性档案令牌 | 冻结参数 |
| specs/macos-parity.toml | macOS 档案令牌 | 冻结参数 |
| specs/effects-catalog.toml | 11 效果槽 × 44 预设 | 槽位定义 |

---

## 1. 设计语言总纲(一句话与三支柱)

**Vitro = "原生质感的克制" × "可玩的插件生态" × "工业级稳定性"。**

- 质感跟 macOS(Liquid Glass 四层材质,25 年交互骨架不变)——卖"以假乱真"
- 可玩性跟 BitDock/Rainmeter(插件目录/皮肤/双形态)——卖"表达力"
- 稳定性是差异化空档(竞品差评主因=崩溃与加载慢,零 hook 架构天然免疫)——卖"永不崩溃、秒开"

设计哲学三原则(源自 Apple HIG + Skillry + transitions.dev 三方交叉验证):
1. **Deference 让位于内容**:无外描边无外投影,层级全靠材质;UI 服务内容不抢戏
2. **品味=一致性与策展**:一个标准贯穿全部素材;先承诺方向(排版声/色彩故事/布局构思/动效风格)再写代码
3. **交互骨架恒定,材质定期换血**:放大抛物线/Genie/运行点 25 年不变;Aqua→Big Sur→Sonoma→Tahoe 四代只换材质

---

## 2. 材质规范(唯一权威数值)

### 2.1 四层结构(Sonoma 基线,深/浅两套)
| 层 | 实现 | 深 | 浅 |
|---|---|---|---|
| ① 背景模糊 | DWM Acrylic(原生)/ backdrop-filter(Web) | blur 22pt + saturate +8% | blur 20pt + saturate +10% |
| ② 基础填充 | 半透明 tint | black @25% | white @18% |
| ③ 颗粒纹理 | 128px 噪点 bitmap WRAP 平铺 | alpha 0-6/255(~0.25%) | 0-7/255(~0.3%) |
| ④ 边缘光影 | 1px 内高光 + 1px 内影 | top white 15-22% / bottom black 12-15% | top white 30% / bottom black 10% |

**全局铁律:无外描边、无外投影**——层级全靠材质;圆角=总高/2;材质不是装饰是深度线索(Depth)。
原生现状(v3):亚克力生效时 tint 打 0.42 折;噪点/光影已实现;待校:hi 0.32→0.16 等基准对齐(工作区已有未提交改动)。

### 2.2 Tahoe 悬浮变体(2025 Liquid Glass,主题可选)
fill 9%/12% · blur 26/28pt · saturate +15/+12% · top 45%/22% · bottom 6%/20% · **0.5px 内折射边**(white 5%,inset 1.5px)· 整体抬升(bottom 24px)。主题切换 = theme.toml 档位,动效档案无关。

### 2.3 原生 API 选型(定案)
dock 药丸 = **Acrylic**(瞬态,blur-behind);偏好/设置窗 = **Mica**(长驻);自绘层 D2D(四层)+ DComp 合成;禁 WebView(红线)。
Web 原型:backdrop-filter(仅模糊)+ feTurbulence 噪点;折射实验用 feDisplacementMap,一屏限一个玻璃元素(多次全表面光栅化红线)。

---

## 3. 几何与排版规范

### 3.1 几何
- 图标:默认 **48pt**(可调 16-128),squircle 圆角 = 边长×20%,写实渲染(渐变+高光+投影)
- Dock:padding 16/10 · 图标间距 8 · 圆角 = 总高/2(两端半圆)· 高度 ≈ 图标+20(48pt→68)
- 分隔:2 条 1pt 半透明线(20%),划分 常驻区|最近区|堆叠/系统区
- 网格纪律:**全部间距 = 4px 的倍数**;容器间 gutter 4px
- 运行指示:4pt 圆点,距图标底 6px,深色 white60%/浅色 black40%;增强对比时增大
- 角标:最小直径 16px,位数自动加宽,右上偏移 (-4,-4),P3 红+1px 内高光,Semibold 11px 白字

### 3.2 排版(Win11 type ramp × macOS 等价,插件/面板直接用)
| 级 | px/行高 | 用途 |
|---|---|---|
| Caption | 12/16 | 标签/元数据/药丸数值 |
| Body | 14/20 | 内容行 |
| Body Large | 18/24 | 强调数值 |
| Subtitle | 20/28 | 卡片标题 |
| Title | 28/36 | 大标题 |
- 数字/时间一律 **tabular figures**(防抖);仅超链接/交互点用 accent;标题可用 subtle 区分
- 排版声(typographic voice)是主题四要素之一:先定声再写码(Skillry)

---

## 4. 动效规范(双档案 + 效果槽)

### 4.1 双档案(用户可切,令牌冻结)
| 项 | macOS 克制档案 | Vitro 弹性档案 |
|---|---|---|
| 签名缓动 | `cubic-bezier(0.25,0.46,0.45,0.94)` 无回弹 | `cubic-bezier(0.22,1,0.36,1)` |
| 弹簧 | — | `cubic-bezier(0.34,3.85,0.64,1)` |
| 放大 | in 200/out 250ms,幂 2.2,follow 抛物线 | follow 160/settle 480ms,falloff 0.45 幂 1.6 |
| 放大峰值 | 默认 1.2(1.0-2.0),cap 128pt | peak 1.6,radius 120px |
| 放大位移 | 向上 = 高度增量/2(底对齐);侧边向屏幕外 | 同 |
| 启动弹跳 | 两跳 12/6pt,0.8s,阻尼 0.7/刚度 100;关=亮度+10% | Boing 两跳(规格 R5) |
| 最小化 | Genie 600 / Scale 500 / Suck 400(隐藏参数);还原=完全逆放 | 同三槽 |
| 自动隐藏 | 显示延迟 300ms/隐藏 500ms/滑入 200ms+渐显 | 隐藏 700ms |
| Tooltip | 延迟 200ms,rise 6pt,in 150/out 100,圆角 8,距图标 10px | 延迟 80ms |
| 堆叠级联 | fan 30ms / grid 20ms | — |

### 4.2 效果槽(11 槽 × 44 预设,effects-catalog.toml)
放大联动/启动/注意力/抽屉/菜单/Tooltip/倒影/3D 倾斜/确认反馈/数值徽章/Dock 入场;每槽带出处标注、性能层级(gpu/raster/js)、**拔电降级目标**。
守卫:prefers-reduced-motion → 全部 0.2s 淡入淡出(最高优先级);电池感知降级;同元素同时只跑一个槽。
原生实现注意:动效定时器只在有动画时挂载,静止即撤(架构决策 3)。

---

## 5. 图标与插件规范

### 5.1 图标系统
- 默认写实包(SQ 25 枚,squircle+渐变+高光+投影;管线 = HTML SVG → 无头 Chromium 96px → include_bytes!)
- **插入即用自身图标**(规格 R6 强化):exe/lnk → `SHGetFileInfoW`/`IExtractIconW` 256px → PNG atlas 缓存 + LRU;URL → favicon
- **图标遮罩**(MyDockFinder 借鉴):非规则图标统一 squircle 裁剪+底板渐变+可选遮罩——解决混插风格混杂(§4.4)
- 动图图标(GIF+alpha)为 BitDock 借鉴项,列主题引擎 M5+

### 5.2 插件(docklet)三层规格
- **三形态**:icon+badge(角标/数值)/ icon+sparkline(线 2px accent 单色,60 点环形缓冲)/ icon+mini-dial(Roundline 环,环厚=R×0.25,三段阈值色)——不开指针拟物
- 数据可视化词汇(Rainmeter 十种 meter)= 官方词汇表;直方图/折线不画网格线
- flyout = medium 卡语言:16px margin、48px 署名区、4px gutter、Caption/Body 两级字阶、WCAG AA 4.5:1(大字 3:1)、**背景图去饱和否则换字色**、头像 96/48/32/24、标题截断用 tooltip
- 对比度按**玻璃合成亮度**校验:L<0.35 白字 / 0.35-0.6 白字+1px 柔影 / >0.6 深字
- 尺寸=内容策略:磁贴 1 触点,flyout 2-3,完整面板 3-4;更多操作走右键菜单(R7)
- 逐插件规格表见 plugin-design-language.md §7(11 个 docklet 逐个落)
- 插件开发者三句话:一眼可读 / 4px 网格 / 触点封顶
- API(boa v3 现行):`api.set(html)` / `api.every(ms,fn)` / `api.log`;正式版 QuickJS 沙箱 + manifest 权限(PLAN §4.3)

---

## 6. 色彩规范

- **单强调色纪律**(Skillry/Cadence):深底 + 恰一个 accent("a charcoal ground, one cyan accent");accent 只给交互点/超链接/选中态
- 深/浅双主题各自独立设计(不是反色);跟随系统亮暗
- **动态主题色**:壁纸取样 → 饱和度加权平均 → HSL 提亮(l=0.5 浅/0.64 深,s=max(0.45,min(0.85,Δ×1.5)))——已实测(#7a82cd)
- 角标红 = P3 系统红 + 1px 内高光;阈值报警色(蓝→红)用 IfAbove 模式逐帧切笔刷
- 背景图入面板必须去饱和(对比度红线)

---

## 7. 交互骨架(25 年恒定签名 × NeXuS R 规格对照)

恒定签名(四代 macOS 不变,行业母语):放大抛物线联动 · Genie/缩放最小化 · 运行指示点。
行为基线 = NeXuS R1-R12(逆向实证),映射:
R1 四边停靠+控制磁贴 ✅ · R2 放大可关 ✅ · R3 Tooltip ✅ · R4 双击 ✅ · R5 Boing+呼吸点(CPU 频率)✅ · R6 拖放全套(拖入/重排/回收站烟雾)✅ · R7 菜单树全量 ✅ · **R8 子码头 = 最大未实现项** · R9 自动隐藏全家(bump/角落/Swipe/热点)✅ · R10 效果开关 8 项 ✅ · R11 持久化(自愈管线)✅ · R12 全家(Shelf/任务/托盘/docklet 群/锁定/折叠)✅
交互红线(不做):接管 Explorer、Dockmod 式辅助组件、轮询输入、管理员敏感路径。

---

## 8. 反模式清单(设计 review 的 ban list)

**视觉反 AI 味**(Skillry):居中布局 / 默认字体(Inter/Roboto/Arial/Space Grotesk)/ 淡蓝按钮 / 平庸 hover / emoji 当图标 / 灰对灰文字 / 只靠 hover 传达 / 全部转场同一时长 / 动画 width-height(应用 transform)/ 无 reduced-motion。
**架构红线**(踩坑实证):ResizeBuffers 动态改交换链 / 全局局部坐标直比 / 常驻定时器轮询 / 绕过 config.rs 手写 TOML / AGPL 代码混入 / boa 0.18 / 双实例测性能。
**体验反模式**(竞品教训):开机逐个加载无统一入场 / 管理员模式功能敏感 / 网盘提取码式分发 / VIP 墙挡核心可读性。

---

## 9. 竞争定位(最终主张)

| 维度 | Vitro | MyDockFinder(¥24) | BitDock(免费+VIP) | Seelen UI(AGPL) |
|---|---|---|---|---|
| 架构 | Rust+D2D/DComp,零 hook | D3D11+Dockmod 接管 | 私有引擎 | Tauri WebView |
| 实测 | CPU 0.39%/RAM 4.14MB | 未知,核显泄漏归咎常见 | 开机加载慢 | 70-120MB 级 |
| 稳定 | 自愈配置+固定画布(零崩溃面) | 623 open issues | 偶发 | 活跃但重 |
| 插件 | boa 沙箱+粘贴即插件+目录 | IM 原创算法/进度条 | 50+ 插件 | Widget SDK(Svelte) |
| 生态 | 复刻配置码(轻分享) | Steam 工坊主题 | BitSkin 皮肤站 | 社区主题 |

**营销三句**:永远不卡(空闲 0%)· 永不崩溃(零 hook 自愈)· 三秒上手(复刻码粘贴即得)。
分发:MS Store + winget + GitHub 签名直发。付费参照:¥24 买断已验证 / VIP 插件墙已验证 / 配置码分享 = 更轻的工坊。

---

## 10. 未解问题与下一步(按优先级)

1. **玻璃质感收尾**:工作区已有噪点+光影校准改动,PrintWindow 视觉验证后提交(流程 §5 HANDOVER)
2. **R8 子码头**(行为规格最大缺口):hover 展开,磁贴簇 + 05-dropdown 原点生长动效
3. **dock 内拖放重排 + OLE 拖出**(R6 残项):WM_DROPFILES 已有,补内部重排
4. exe 体积 9.66MB→≤5MB:boa feature-gate 或 WASM 沙箱迁移
5. 多显示器(input.rs screen_h 只认主屏;固定画布在副屏 DPI 需重审)
6. `>` 命令输出捕获 / 启动器 IME 中文
7. M5 主题引擎:四要素声明(typographic voice/colour story/layout idea/motion style)+ 昼夜感知换肤 + 裸排版主题(Rainmeter 极简文字派,最大审美人群)
8. 设计 review 技能(Impeccable 式 audit/polish/animate 对照本总纲)
9. 远期:Vitro Dock MCP(让 agent 操作 dock)、桌面画布 widget 形态(与 Shelf 二选一)

---

## 11. 验证与品控(全流程)

- 视觉:PrintWindow(PW_RENDERFULLCONTENT)抓 DComp → 深底合成 → 与 HTML 基准对拍(tools/capture.py 已固化)
- 性能:8s 采样 CPU<1%/RAM<30MB;构建前必杀旧实例
- 行为:selftest Frida 自证(0 轮询证据)
- 设计 review:本文 §8 ban list + plugin-design-language §7 逐插件表 + macos-parity 令牌比对
- 交付纪律:自己先用 PrintWindow 亲眼确认再报告(HANDOVER §7)
