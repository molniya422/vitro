# Nexus Dock 核心交互 · 行为规格 v0.2

范围:Dock 条核心交互(复刻选项 2)。不含皮肤引擎、内嵌模块(天气/邮件/CPU)、更新机制。
证据源:A = Nexus.exe 内嵌 UTF-16 字符串(4321 条候选);B = History-nx.txt 官方更新日志;C = 安装目录文件;**D = Frida 运行时捕获(nexus-startup-evidence.json,spawn 模式 12 秒启动序列,3 轮)**。标注:【C】= CONFIRMED、【I】= INFERENCE、【U】= UNKNOWN。

## R1 结构与定位
- Dock 停靠四边之一:上/下/左/右(A: "Dock to Top/Bottom/Left/Right Screen Edge");左右边时为垂直方向,上下边为水平(A: "Vertical"/"Horizontal", "Position the dock in which orientation?")【C】
- Dock 通过 Win32 AppBar 协议保留屏幕边缘空间(A: "NEW APPBAR POS NOTIFICATION", "Reseting Reserved Screen Space", "New Windows Taskbar Created")~~【C-行为存在;API 调用点 U】~~ → **v0.2 修正:运行时证伪。SHAppBarMessage 三轮捕获零调用**。任务栏协调改以 FindWindowW("Shell_TrayWnd") 定位任务栏窗口(D,启动期 1 次)+ always-on-top 窗口策略实现(保留屏幕空间的机制仍未定位,U;推测为自管 Z 序 + 最大窗口避让,B 的 auto-hide for maximized 行为佐证)【D】
- 左端有控制磁贴(control tile):单击开 Preferences,双击折叠/展开 Dock(A: "Left clicking the control tile opens %1 Preferences", "Double clicking the control tile collapses/uncollapses the dock")【C】
- 支持 Always On Top / Always On Bottom(A)【C】

## R2 图标放大(magnify)
- 悬停产生邻近图标联动放大(高斯型衰减),可整体关闭;关闭时仅单图标放大且标签位置随之调整(B: "…for docks not using the magnify effect so the label does not 'sit on the edge'")【C-存在;曲线参数 U,原型按常见实现假设】
- 放大时如启用反射,反射不随放大重建(性能优化,B: "not rebuilding the reflections when the icon is magnified")【C】

## R3 标签与 Tooltip
- 悬停显示图标标签;Tooltip 由定时器驱动(A: "In Tooltip Timer", "Tooltip", "Show in Tooltip")【C-存在;延迟毫秒数 U,原型取 500ms】
- 点击预览类条目有 "Click to Preview" 文案(A)【C】

## R4 点击/双击时序
- 双击由定时器窗口判定(A: "In Double Click Timer", "DoubleClick")【C;窗口宽度 U,原型取 400ms】
- 拖拽起始有 6px 级阈值判定(A: "Drag Start", "Start Drag!")【I】

## R5 启动效果与运行状态
- 启动条目播放 Launch Effect(弹跳类;设置中有 "Disable Boing Effect",B)【C】
- Dock 可显示运行中程序的图标与指示(A: "running applications in this dock")【C;指示样式 U,原型用光点】

## R6 拖放
- 图标可拖拽重排(A: "Start Drag Event", "Entering Drag Over", "OLEComplete Drag Effect")【C】
- 拖到回收站图标上删除该条目(A: "Drop Item into Recycle Bin")【C】
- **OLE 放置目标(运行时证实)**:注册 UniformResourceLocator(W)/FileGroupDescriptor(W)/Shell IDList Array/Object Descriptor/Preferred DropEffect/HTML Format 等 64 种剪贴板格式(D)→ 可从资源管理器拖入文件、从浏览器拖入 URL 新建条目;配套 Drop/Drag Effect 设置(A)【C-存在;细节 U】

## R6b 运行时架构(D,三轮 spawn 捕获一致)
- 输入模型:GetCursorPos 连续轮询 40–68 次/秒(≈15–25ms 间隔),驱动放大与边缘检测;**无 SetWindowsHookEx 鼠标钩子**
- 任务扫描:EnumWindows 1 次/秒(与成组的 1000ms SetTimer 心跳一致)→ 运行应用指示的刷新周期为 1s【D】
- 模块刷新定时器群:10s/30s/60s(天气/时钟等模块各自心跳);Tooltip/双击定时器**不在启动期创建**(交互期懒创建,U)
- 单实例:未见 CreateMutexW;推测 FindWindow 按窗口定位(I)
- 启动期零注册表访问、零 SHAppBarMessage(D)

## R7 右键菜单
条目菜单(A+B,顺序按 changelog "Pin to Dock has been moved to the top"):
Pin to Dock → Launch Item → Run As Administrator → Run as Another User → ── → Rename ── Remove from Dock ── Item Properties("Dock Entry Properties"/"%1 Properties")
回收站条目显示 "Recycler Properties"(A)【C】
Dock 空白菜单(A):Insert New Dock Item / New Menu(子码头)/ New Folder / Duplicate Sub-Dock / Detach Sub-Dock / Position 子菜单(四边)/ Disable-Enable Auto-Hide / Effects 子菜单(Icon Reflections, Boing…, Delete Effect 类别)/ Dock Properties / Preferences("Show Dock Properties", "Manage/Management")【C-条目存在;分组归属 I】

## R8 子码头
- 文件夹/菜单条目可展开为子码头(A: "Open Sub-Dock", "Sub-Dock Properties", "New Menu")【C;展开动效 U】

## R9 自动隐藏与边缘触发
- 自动隐藏可开关(B: "Disable/Enable Auto-Hide"),针对"最大化窗口 + Always On Top"有独立子行为(B)【C】
- 显示触发方式:鼠标碰边/滑边 + 屏幕四角 bump(A: "Bump/Swipe Top/Bottom/Left/Right Screen Edge", "Bumping … Screen Corner" ×4)+ 可配置热点点击(A: "Left/Right/Middle Clicking on Hotspot")【C】
- 激活与隐藏有速度/动画设置(B: "Animation Speed slider", "Animation type")【C;时长 U】

## R10 视觉效果(开关项,A 全部实测到字符串)
Icon Reflection / Fluid Reflection / Fluid Surface / Background Blur / Water Ripple / Icon Drop Shadows / Mouseover Effect / Attention Effect【C-开关存在;渲染算法 U】

## R11 持久化
- 主题与背景以 INI 文本存储(A: "Background.INI", B)【C】
- **配置根目录已运行时定位:`C:\Users\Public\Documents\WinStep\`**——`NeXuS\Backgrounds\SDVZ4\SDV.png`(当前主题背景)、`Icons\Samples\*.png`、`Languages\NeXuS\Chinese Simplified.ini`(激活语言,读 2 次)/English.ini 等 20+ 语言包(D,启动期 File 句柄)【D】
- **启动期零注册表访问**(RegOpenKeyExW/RegCreateKeyExW 过滤 winstep|software 均无命中)→ 早期 UNKNOWN"设置存哪里"的答案:文件树而非 HKCU 注册表(至少加载路径如此;设置保存路径一致性 U)【D】

## R12 新发现功能域(v0.3 附录,2026-10-24,证据 = 全量 4321 条 UTF-16 字符串二次挖掘)
- **Shelf 搁架**:独立于 dock 的 Shelf 对象(A: "Insert New Shelf Item", "Shelf Entry Properties", "Displaying Shelf", "Make Shelf Tab from Dock", "Saving Shelf Info")【C-存在;布局 U】
- **任务列表模块**:运行窗口列表可入 dock,支持分组/定制图标/排除(A: "Display a Menu Showing the Currently Running Applications", "Tasklist Grouping", "Add Customized Task Icon", "Add Excluded Task", "Add Task to Thumbnail Exclusion List", "Only show this task when the window is minimized")【C】
- **系统托盘嵌入**(A: "DockShowSystray", "DockShowFullTray", "DockGroupTrayIcons", "Show/Hide Systray Icon", "System Tray Expand Button")【C】
- **时钟 docklet**:模拟/数字双模式、原子钟网络对时、整点报时与语音主题(A: "Analog/Digital Clock Background", "Check Atomic Clock", "Clock was %1 seconds behind", "Clock Chime/Tick", "ClockVoiceTheme*")【C】
- **天气 docklet**:METAR 多源(weather.com/noaa)、预报面板(A: "METARShowForecast", "Retrieving Weather", "Detailed Weather Conditions")【C】
- **邮件检查器**:最多 9 个 POP3 账户、未读提醒音、启动客户端(A: "of 9 separate email accounts!", "a002 STATUS Inbox (MESSAGES RECENT UNSEEN)", "Email Checker Chime")【C】
- **CPU/RAM/网络表**:直方图/仪表两种背景、CPU 大户提示、文字样式定制(A: "Histogram/Gauge Meter Background", "Current CPU hog:", "CPU Usage: %1%")【C】
- **电池监视器**:低电/危急两级告警音、AC 切换事件(A: "PSMBatteryLow/Critical", "wsBatteryLow.wav", "BatteryPowerRestored")【C】
- **注意力效果**:可开关 + 到达时激活 dock(A: "DockAttentionFx", "DockActivateOnAttention")【C】
- **水波纹**:速度/尺寸/距离三参(A: "RippleSpeed/Size/Distance", "DockWaterRipple")【C】
- **玻璃模糊分级**:全局/菜单/dock 各自模糊度(A: "GlassBlur", "DockGlassBlur", "MenuGlassBlur", "GlobalGlassBlur")【C】
- **桌面热点按钮**:桌面上独立的 Hotspot 按钮,左/中/右键各自可配(A: "Hotspot Button on the Desktop", "Left/Middle/Right Clicking on Hotspot")【C】
- **Swipe 滑边激活**:与 bump 并列,距离+时限判定(A: "SwipeDistance", "SwipeTimeLimit", "DockSwipeActivation")【C】
- **启动时折叠**(A: "DockCollapseOnLaunch", "SubDockCollapseOnLaunch")【C】
- **密码锁定**(A: "Please Enter The Password!", "Password Properties")【C】
- **窗口缩略图预览**(A: "Validating Thumbnail", "TaskbarSnapshotSize", "Retrieve thumbnail once but do not refresh it")【C;渲染细节 U】
- **垂直 dock 无反射**(A: "DockVertNoReflection")【C——行为规则,原型已遵循】
- **控制磁贴右键 = dock 菜单**(A: "Right clicking the control tile displays the dock context menu.")【C——R7 补充】

## UNKNOWN 汇总(v0.2 更新)
1. 放大曲线参数与最大倍率(原型:峰值 ×1.6,σ≈70px)— 需视觉对拍
2. Tooltip/双击定时器毫秒值 — 启动期不创建;**测法:frida -p <pid> attach 后手动悬停/双击,SetTimer/GetDoubleClickTime 即现身**(运行时挂接已就绪,D:\rea\nexus-hooks.js)
3. 自动隐藏动画时长与缓动 — 需交互对拍
4. ~~AppBar 调用点~~ **已证伪**;保留屏幕空间的真实机制(推测自管 Z 序)— 需交互期捕获 SPI/WM 与窗口样式
5. 运行指示具体视觉样式 — 需视觉对拍
6. SPI action 66(0x42)与 4130(0x1022)用途 — 低相关,未解
7. 菜单图标与分组细节 — 需视觉对拍
