# 调研:Windows 桌面美化软件"插件"的前端设计语言(细化版)
> 2026-10-24 v2 · 三层结构:**风格(长什么样)→ 设计思路(为什么这么设计)→ 具体实现(参数级怎么做)**
> 证据源:Microsoft Learn 小组件设计规范(2026-08 全文)、Rainmeter 官方 System 教程(参数级)、Aero Gadgets 史料、XWidget/ObjectDock/SysStats、Vitro 逆向规格 R12
> 关联:design-language-research.md(宏观)、HANDOVER.md(v3 现状:插件磁贴 = SQ 图标 + 数值药丸)

---

## 0. 四种形态与三代源流(浓缩)

形态:A. dock 内插件(docklet)· B. 桌面小组件(widget)· C. 弹出面板(flyout)· D. 顶栏组件。
三代源流:Aero Gadgets(拟物玻璃,2006)→ 皮肤引擎(Rainmeter/XWidget,词汇表)→ Fluent 卡片(Win11,数值规范)。详见 v1 版本与来源节。

---

## 1. 数据圆环/仪表(Roundline 词汇)

### 1.1 风格
- 圆弧环绕图标或独立成环:深色底环 100% 常驻,亮色前景弧随数值增长;中心放数值或标签;CPU 高时文字变红
- 气质 = "仪表盘的一角":克制、单色、无渐变、无 3D——所有信息在 2px-5px 的环厚里

### 1.2 设计思路
- **底环 + 前景弧双层**:底环给"满量程预期",前景弧给"当前值"——人眼对"缺口长度"的读数快于对数字的读数(glanceable 的圆环实现)
- 中心数值冗余校验:弧给趋势感,数字给精确值,二者叠加 = 一眼 + 一瞥两级读数
- 条件变色(阈值越限变红)是仪表盘语言的"报警灯",替代新增报警组件

### 1.3 具体实现(Rainmeter 参数级 → Vitro D2D 等价)
Rainmeter 教程原参数(双 Roundline 叠加):
```
[MeterCPUCircleBack]              ; 底环:不绑 Measure → 恒满圆
Meter=Roundline / W=40 H=40
StartAngle=(Rad(270))             ; 0°=正右方,Rad(270) 转到"顶部"为起点
RotationAngle=(Rad(360))          ; 100% = 一整圈
LineStart=15 LineLength=20        ; 内半径 15 / 环厚 5(40px 圆半径 20)
Solid=1                           ; 填充弧而非指针
LineColor=#MediumBlue#

[MeterCPUCircle]                  ; 前景弧:绑 MeasureCPU,其余同构
MeasureName=MeasureCPU
LineColor=#LightBlue#             ; IfAboveValue=24 → IfAboveAction 变 #LightRed#
```
中心数字:`Meter=String / FontFace=Segoe UI / FontSize=8 / StringAlign=CenterCenter / X=26 Y=28 / Text=%1%`(圆心坐标 + CenterCenter 对齐)。

**Vitro D2D 等价实现**(原生 M2 的落点):
- 底环:`rt.DrawEllipse(中心, 半径R, 底环笔刷, 环厚W)`(D2D1 接口画整椭圆)
- 前景弧:`D2D1_STROKE_STYLE` 不直接支持部分弧——用 `CreatePathGeometry`:`FigureBegin` 在 `角=-90°` 处 `AddArc(D2D1_ARC_SEGMENT { point: 中心+R*向量(角), size:(R,R), rotationEnd:0, sweep: CLOCKWISE, arcSize: value>50%?LARGE:SMALL })` 到 `角=-90°+value*360°`;然后 `DrawGeometry(弧路径, 环厚W, 前景笔刷)`
- 参数换算:`环厚 = R * 0.25`(40px 图标环厚 5px 的比例);起始角固定 -90°(顶部,与 Rad(270) 等价);sweep 角 = `value * 2π`
- 阈值变色:每帧比较 `value`,超阈切红笔刷(零成本,无需 IfAction 机制)
- 中心数字:DWrite `IDWriteTextLayout` 预格式化(StringAlign=CenterCenter = layout 按 textAlign CENTER + paraAlign CENTER 定位到圆心);数字用 tabular figures 防跳动

### 1.4 变体:指针表盘(Aero 时钟/Rotator 词汇)
- **风格**:玻璃表盘 + 双指针 + 投影;Windows 7 时钟的实际构造 = **分层 PNG 叠加**(表盘底图、玻璃外圈高光、时针/分针各一层,指针层绕中心旋转变换,再加软方向阴影)——不是矢量重绘
- **思路**:表盘是"装饰恒定 + 数据旋转"的分离结构;装饰一次成图,每帧只动数据层
- **实现**:Vitro 时钟 docklet 已是矢量版(`clkHands` 三针重绘,SVG→SQ 内嵌);原生等价 = 表盘 bitmap 固定 + 指针 bitmap `SetTransform(旋转矩阵, 中心)` 每分钟一次;投影 = 指针 bitmap 先以黑色 30% 偏移 (2,3) 画一遍再画本体

---

## 2. 火花线/直方图(Histogram·Line 词汇)

### 2.1 风格
- dock 磁贴内:图标下方 2px 高的微型走势线,或磁贴底部 5-7 根 2px 柱(Nexus RAM 表 = 直方图,规格 R12 证据 "Histogram Meter Background")
- 气质 = "示波器的角落":无轴、无边框、单色、靠形状本身读数

### 2.2 设计思路
- sparkline(火花线)的定义就是"不坐标轴化的小型趋势图"——省掉轴/刻度/网格,数据形状即全部
- 直方图 vs 火花线:柱状适合"分段读数"(每核 CPU),线适合"趋势读数"(网速);docklet 空间 < 60px 时线 > 柱
- 采样窗口即叙事:60 个采样点 = "最近一分钟",窗口长度决定"时间感"

### 2.3 具体实现
Rainmeter 参数:
```
[MeterCPULine]
Meter=Line / MeasureName=MeasureCPU / W=130 H=38
LineCount=1 / LineWidth=1
HorizontalLines=1 / HorizontalLineColor=#AlmostBlack#   ; 网格线
SolidColor=#DarkBlue#                                    ; 图底
```
直方图(Histogram meter):柱宽/间隙参数 + 双色(高低阈值分色)。

**Vitro D2D 等价**:
- 环形缓冲 `Vec<f32>`(容量 60),采样器每 500ms push(UpdateDivider 思路:低频数据降采样率省 CPU)
- 火花线:`PathGeometry` 连线 `点(i) = (x0 + i*步长, baseline - 值*高)`;`DrawGeometry(线宽 2.0, 尖头/圆头 LineJoin)`;**不要网格线**(磁贴内网格=噪音)
- 直方图:`FillRectangle` × N(柱宽 2px、间隙 1px、底部对齐);超阈柱切 accent 红
- 填充渐变(可选):`CreateLinearGradientBrush`(值色 → 透明)做柱体渐隐,模仿 "Histogram Meter Background" 的景深感
- 60 采样点 × 每 500ms = 内存 ~240B/插件——零预算压力;**采样定时器只在磁贴可见时挂载**(遵守架构决策 3:静止即撤)

---

## 3. 角标与数值药丸(Badge 词汇)

### 3.1 风格
- 右上角红色圆药丸 + 白色数字;或图标底部深色药丸 + 时间/数值(Vitro 现行:时钟 12:34)
- 气质 = "图标本体的延伸",不加边框不投影(规格 R5/R12 与 Win11 一致)

### 3.2 设计思路
- 角标回答"要不要看",数字药丸回答"是多少"——两者是图标的注脚,不是独立 UI
- 最小直径 16px、位数增加自动加宽(macOS 规格)保证两位数不换行;数字等宽(tabular)防"时钟抖动"

### 3.3 具体实现(数值全部来自官方/规格)
| 项 | 参数 | 来源 |
|---|---|---|
| 角标最小直径 | 16px,内边距水平 4px | macOS 规格表 |
| 角标字号/字重 | SF Pro Bold 12pt 白字 → Segoe UI Semibold 11px 等价 | macOS 规格表 |
| 角标红 | P3 系统红 + **1px 内高光**(顶部白色 45%)→ CSS `linear-gradient(180deg,#ff6a6a,#e8342e)+inset 0 1px 0 rgba(255,255,255,.45)` | macOS 规格表/原型 |
| 药丸圆角 | 全圆(高度/2) | 原型现行 |
| 位置 | 右上偏移 (-4,-4);呼吸点在图标下方 6px 处 4px 直径 | macOS 规格表 |
| 弹入动画 | scale .4→1,spring `cubic-bezier(0.34,1.36,0.64,1)` 500ms | t.dev#03 |
| 数字字体 | **tabular figures**(DWrite:数字字形等宽),时间 HH:MM 不闪动 | Win11 字阶纪律 |
| 未读清零 | 点开 flyout 即清零 + 角标 spring 回缩 | 原型现行 |

**Vitro 实现**:plugin.rs 的 `api.set` 输出进 `plugin_fmt_sm`(DWrite 小号格式化);原生角标 = `FillRoundedRectangle(药丸, radius=高/2) + FillRoundedRectangle(顶 1px 内高光条)`;宽度按文字测量 `DrawTextW(TEXT_CALCRECT 等价)` 自适应。

---

## 4. 弹出面板/卡片(flyout,Fluent medium 卡语言)

### 4.1 风格
- 玻璃卡片从触发磁贴方向弹出;信息两级(标题行 + 条目行);分区分隔用 4px gutter + 1px 分隔线
- 气质 = "dock 的放大镜":同材质、同字号体系、同 accent,只是面积放大

### 4.2 设计思路
- flyout = medium 卡尺寸策略:2-3 个触点封顶;更多操作跳完整面板(收敛交互)
- 排版两级就够:Caption(标签/元数据)+ Body(内容)——Win11 规范的 5 档字阶在卡片里只用到 2 档
- 尺寸=内容策略也适用于弹出:预报 3 行就是 3 行,不填满

### 4.3 具体实现(Win11 官方数值,全部可直接搬)
| 项 | 数值 | 备注 |
|---|---|---|
| 外边距 | **16px**(四周) | 内容禁出 |
| 署名区 | **底部 48px** 禁放内容 | 对应我们面板的"状态行" |
| 元素间距 | **4px gutter**,容器贴 margin 边 | |
| 间距纪律 | 全部 **4 的倍数** | 跨分辨率像素干净 |
| 字阶(Segoe UI,px/行高) | Caption 12/16 · Body 14/20 · Body Strong 14/20 Bolder · Body Large 18/24 · Subtitle 20/28 · Title 28/36 | 卡内用前两档即可 |
| 色彩 | 标题/正文 = 主题默认色(+subtle 区分);**accent 只给超链接/交互点** | |
| 对比度 | 正文 ≥4.5:1,大字/图形 ≥3:1(WCAG AA);**背景图必须去饱和**否则换字色 | 官方正反例 |
| 背景 | 纯色/渐变 tint/图片 三选;浅深两套独立设计 | |
| 头像 | 96/48/32/24 四档圆形 | 邮件面板发件人头像 |
| 截断 | 标题截断用 tooltip,正文不截断 | DWrite ellipsis + 现有 tooltip 机制 |

**Vitro 实现**:flyout 现为 HTML 原型 `.fly`;原生 M2 = D2D 圆角矩形玻璃卡(同药丸材质,blur 半径同)+ DWrite 排版;邮件面板 = Body 行(发件人 Body Strong)+ Caption 行(时间/摘要),行高 20px + gutter 4px;头像 24px 圆形(裁剪椭圆 clip)。

---

## 5. 桌面小组件(B 形态,Win11 三尺寸策略,M5+ 参考)

### 5.1 风格与思路
- 三尺寸 = 三份独立内容设计,不是缩放:small=单一信息(glanceable 极致),medium=2-3 触点,large=3-4 触点或沉浸单图
- 背景:纯色/渐变/图片三选;与系统主题联动(浅深各一份)
- 这是"内容策略API化":尺寸选择即内容选择

### 5.2 实现参考(将来 ui.toml 模板化)
- 每插件声明三档布局:`ui.toml` 按 size 分节(类比 Adaptive Cards 模板按 size 切换),渲染器按 dock/桌面画布档位取对应节
- Vitro 落点:与规格 R12 的 Shelf 二选一(M5 决策);若做,复用 flyout 排版器 + 48px 署名区放插件名

---

## 6. 材质跟随宿主(docklet 的对比度纪律)

### 6.1 思路
docklet 没有自己的背景——它的"背景"就是玻璃药丸,所以:
- 文字对比按**玻璃实际合成亮度**计算,不是按理论 tint
- 玻璃加深(theme.alpha↑)→ 白字对比↑;亚克力背板亮壁纸下玻璃变亮 → 白字对比↓——**必须给文字加"可读性阴影"或最小底板**

### 6.2 实现
- 实测法:渲染药丸时把 tint+亚克力合成后的平均亮度 L 算出来(对药丸 rect 采样或直接由 alpha/tint 推导:`L ≈ tint_lum*(1-a) + backdrop_lum*a`,backdrop 未知时取保守 0.5)
- 规则:`L < 0.35 → 白字 100%`;`0.35-0.6 → 白字 + 1px 柔影(black 40%, offset 1px)`;`>0.6 → 深字 #20242e`
- 颗粒纹理层(Sonoma 第三层)已实现(build_noise,alpha 0-6/255)——它在数值药丸上同样生效,玻璃抗色带

---

## 7. 逐插件设计规格表(Vitro 现有 11 个 docklet 逐个落)

| 插件 | 形态 | 数据 | 磁贴呈现(风格) | 动效 | flyout 内容(medium 卡) |
|---|---|---|---|---|---|
| 时钟 | 数值药丸 | 本地时间 | HH:MM tabular,15px;模拟模式=矢量三针表盘(§1.4) | 每秒跳秒/分针平滑 | —(点击切模拟/数字) |
| 天气 | 图标+温度药丸 | METAR | 图标=实时天气,药丸=当前温度 | — | 未来 3 日(fc-day 行,Caption+Body) |
| 邮件 | icon+角标 | 未读数 | 红药丸 16px+,semibold 11px | 弹入 spring;注意力效果联动 | 3 条最近(头像 24px 圆 + Body Strong 发件人 + Caption 摘要/时间) |
| CPU | icon+sparkline | 进程时间 | 2px 线,60 点,accent 单色 | — | 直方图 130×38 + 当前 % Body Large + 大户进程名 |
| RAM | icon+直方图 | 内存 | 5-7 柱,2px 柱宽 1px 隙,底部对齐 | — | 已用/总量 AutoScale("931 GB"式) |
| 电池 | icon+mini-dial | 电量 | Roundline 环,环厚=R*0.25,绿→橙→红三段阈值色 | 低电阈值触发注意力(规格 R12 证据 PSMBatteryLow) | 电量 % + AC 状态行 |
| 任务 | icon+点阵 | 运行窗口 | 图标下 2-3 个 5px 圆点(现行 mini) | 新窗口点弹入 | 窗口清单(点击切换,缩略图 M2) |
| 托盘 | icon+点阵 | 托盘图标 | 同上 3 点 | — | 托盘图标网格(22px,间距 12px) |
| 网速 | icon+双向药丸 | 网速 | ↑↓ 两行 Caption(11px tabular) | — | 下行/上行 Body Large |
| 日历 | icon+日期药丸 | 日期 | 药丸显示"24" | — | 月历网格(7 列,gutter 4px,今天 accent 圈) |
| 剪贴板 | icon+角标 | 历史 | 角标显示条数 | 新条目弹入 | 3 条最近(单行 ellipsis,点击回填) |

## 8. 插件开发者三句话(将来随 boa API 文档发布)
1. **一眼可读**:磁贴只回答一个问题;答案要么是角标数字,要么是形状(线/环),不写字
2. **4px 网格**:所有间距取 4 的倍数;药丸高 = 字号行高 + 8
3. **触点封顶**:磁贴 1 个动作,面板 2-3 个;更多进右键菜单(规格 R7)

## 附:来源
[Widget design fundamentals(MS Learn 2026-08)](https://learn.microsoft.com/en-us/windows/apps/design/widgets/widgets-design-fundamentals) · [Rainmeter System 教程(Roundline/Line/String 参数)](https://docs.rainmeter.net/manual/getting-started/basic-tutorials/system) · [Rainmeter Meters 总览](https://docs.rainmeter.net/manual/meters/) · [Apple HIG Widgets](https://developer.apple.com/design/human-interface-guidelines/widgets) · [Windows Desktop Gadgets 史](https://en.wikipedia.org/wiki/Windows_Desktop_Gadgets) · [GadgetPack](https://gadgetpack.net) · [Gadgets for Rainmeter](https://github.com/SilverAzide/Gadgets) · [fGadget CPU Meter Pro(14 表盘)](https://apps.microsoft.com/detail/9n02j7kbvtbp) · [SysStats](https://www.wincustomize.com/explore/objectdock/4271) · [XWidget](https://www.xwidget.com/xwidget.html) · [macOS 规格参数](macos-dock-spec.md) · [NeXuS R12 证据](nexus-dock-behavior-spec.md)
