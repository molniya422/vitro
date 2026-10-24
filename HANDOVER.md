# Vitro Dock 接手文档

> 最后更新:2026-10-24(v3 重写完成、GitHub 建库后)。写给下一个接手的人(或未来的会话)。
> 读完这份 + `docs/INTEGRATION.md`,你就拥有全部必要的上下文。

---

## 0. 30 分钟上手清单

1. 装好 Rust stable(唯一构建依赖)
2. `cd D:\dockyh\vito && cargo build --release` → `./target/release/vitro.exe`(约 1 分钟)
3. 打开 `docs/prototypes/vitro-nexus-full.html`(浏览器)——这是**视觉与交互的验收基准**,原生实现以它为目标
4. 跑一遍性能门(见 §5),确认数字在预算内
5. 通读本文档 §2(架构决策)和 §3(踩坑记录)——这些坑全部真实发生过

## 1. 项目一句话与当前状态

**Windows 原生 macOS 风格 Dock,Rust + DirectComposition,零轮询事件驱动。**

| 项 | 状态 |
|---|---|
| 仓库 | https://github.com/molniya422/vitro (private,MIT,凭据走 Windows 凭据管理器可直接 push) |
| 里程碑 | M0-M7 全部完成(v3 全量重写后) |
| 性能 | 空闲 CPU 0.59% / RAM 7.09MB(预算 30MB)· exe 9.66MB(⚠ 超 5MB 预算,boa 引擎占 8.5MB,既知) |
| 已实现 | 玻璃药丸/DWM 亚克力 · falloff 弹簧放大 · 高清图标提取 · SQ 图标库 · 倒影 · 运行呼吸点(CPU 频率) · tooltip · 右键菜单 · 拖放加图标 · 启动器(Ctrl+Alt+D) · boa 插件沙箱 · 主题热重载(Ctrl+Alt+R) · 托盘 · 底边召唤自动隐藏 · 电池降级 |
| 未实现 | NeXuS R8 子码头 · dock 内拖放重排 · OLE 拖出 · R10 视觉开关群 · 多显示器 · `>` 命令输出捕获 · 启动器 IME 中文 · 启动弹跳动画 · 自动更新器 |

## 2. 架构:五个不能改错的决策

这些决策都各自解决过一个真实事故,**推翻前先读 §3 对应的坑**:

1. **窗口 = 固定 1024×124 透明画布,永不 resize**(`gpu.rs`)
   玻璃"药丸"按磁贴数在绘制层自适应宽度居中;药丸外像素由 DComp 透明点击穿透。
   v2 曾按磁贴数 ResizeBuffers → `target_bmp` 持 buffer 引用导致 resize 失败级联(图标切边、布局飞出窗口)。v3 让这个 bug 类物理上不存在。**任何"动态改交换链尺寸"的想法都要三思。**

2. **坐标两套,一处换算**(`state.rs`)
   钩子给屏幕物理坐标;渲染用窗口局部坐标。`State::tile_center_local/_screen` 成对出现,`hit_test` 只吃屏幕坐标。v2 曾拿全局坐标直接比局部坐标 → 悬停放大/tooltip/点击全部静默失效。**新增任何命中判断,先确认坐标系。**

3. **零轮询是构造保证,不是优化**(`input.rs`)
   WH_MOUSE_LL → 药丸 ±200px 邻域门控 → 8ms 节流 PostMessage → GetMessageW 阻塞。
   光标远离 = 0 消息 0 渲染 0 定时器。空闲 CPU ≈0% 由此保证。**禁止引入任何常驻定时器/轮询线程**;动效定时器(TIMER_ANIM 33ms)只在有动画要结算时挂载,静止即撤。

4. **插件宿主 = boa_engine 0.17**(纯 Rust,零 C 依赖)
   rquickjs 要 libclang、quick-js 的 C 源码用 GCC 内建,MSVC 都编不过——boa 是试错后的唯一幸存者。**boa 0.18 在 Windows 有 futex 线程安全 bug,不要升级。** `plugin.rs` 在 `STATE` 初始化前执行,**里面绝不能调 `s()`/`app()`**(v3 曾因此启动即崩)。

5. **配置读写必须走自愈管线**(`config.rs`)
   读:解析失败 → 坏文件备份 `.toml.corrupt` + 回默认;路径剥控制字符。
   写:先备份 `.toml.bak`。历史上 TOML 双引号把 `\n` 转义成真换行,坏配置导致用户素材整份消失回退默认(用户非常愤怒)。**绕过 `config.rs` 手写 TOML 的做法一律禁止。**

## 3. 踩坑记录(全部真实发生)

| 坑 | 症状 | 根因/解法 |
|---|---|---|
| ResizeBuffers 失败级联 | 图标切边、磁贴飞出窗口 | D2D target 位图持 buffer 引用;v3 固定画布根治 |
| 悬停/点击全偏移 | 图标呆板无放大、tooltip 不出 | 全局坐标 vs 局部坐标直比;v3 state.rs 统一 |
| 配置损坏素材消失 | 用户 dock 图标/插件全没了 | TOML `\n` 转义链式污染;自愈管线根治 |
| plugin 启动即崩 | `Option::unwrap() on None` | `load_plugin` 在 STATE 初始化前调了 `app()` |
| 双实例污染测量 | "空闲 CPU 8%"假警报 | 前一实例没杀干净;**构建前必 `taskkill //IM vitro.exe //F`** |
| 截图验证失灵 | 进程活着、窗口可见、截图里却没有 | `CopyFromScreen`/`BitBlt(SRCCOPY)` 抓不到 DComp 内容;**必须 PrintWindow(hwnd, hdc, 2=PW_RENDERFULLCONTENT)**,脚本见 §5 |
| rquickjs/quick-js 编译失败 | libclang 缺失 / GCC builtin | 用 boa 0.17 |
|boa 0.18 升级 | Windows 死锁 | 锁 0.17 |

## 4. 代码地图

```
D:\dockyh\
├── HANDOVER.md          ← 本文
├── README.md / LICENSE(MIT) / .gitignore
└── docs/
    ├── INTEGRATION.md   ★ 四方资源取舍权威表(NeXuS/Glassboard/macOS规格/HTML原型)
    ├── PLAN.md          ★ M0-M7 里程碑日志,每步带性能数字和技术笔记
    ├── prototypes/      4 个 HTML 原型(vitro-nexus-full.html 是视觉基准)
    ├── research/        nexus-dock-behavior-spec.md(R1-R12 行为规格,逆向证据)
    │                    macos-dock-spec.md / competitive-research-2026-10.md
    └── specs/           motion-tokens.toml(动效令牌) / effects-catalog.toml / macos-parity.toml
D:\dockyh\vitro\         ← Rust 项目
├── src/
│   ├── main.rs          装配:窗口/画笔/磁贴/热键/消息循环
│   ├── gpu.rs           D3D11→D2D→DComp 一次成型(Gpu 结构体 RAII 保活)
│   ├── state.rs         ★ 唯一 static STATE + 药丸几何 + Tile 模型
│   ├── dockwin.rs       ★ wndproc + render_frame(玻璃/放大/呼吸/倒影/徽章)+ 交互
│   ├── input.rs         ll_hook 邻域门控 + R9 召唤/隐藏 + 电源
│   ├── config.rs        自愈 TOML + Tahoe 主题
│   ├── icons.rs         Shell 三档提取 + SQ png→BGRA 预乘→D2D
│   ├── tray.rs          托盘
│   ├── lpad.rs          启动器(自包含,有自己的 DComp 管线)
│   └── plugin.rs        boa 沙箱(api.set/every/log;plugin_fmt_sm=徽章字体)
├── assets/icons/*.png   SQ 图标 25 枚(96px RGBA)
├── tools/gen_sq_icons.py 图标再生成管线
├── selftest/            Frida 自证(hooks + runner + 证据 JSON)
└── src/bin/perf-harness.rs
用户数据:~/.vitro/dock.toml(磁贴/插件/autohide) + theme.toml(alpha/tint/accent/peak/radius)
```

## 5. 验证手段(改完代码必须跑)

```bash
# 0) 杀旧实例(必须,防双实例污染测量)
taskkill //IM vitro.exe //F

# 1) 构建
cd /d/dockyh/vitro && cargo build --release 2>&1 | grep -A8 "^error"   # 目标:零 error

# 2) 启动 + 看启动日志(tiles/pill_w/backdrop_ok 应合理)
./target/release/vitro.exe > /tmp/vitro.log 2>&1 & sleep 4; cat /tmp/vitro.log

# 3) 性能门(8s 采样;预算:CPU <1%,RAM <30MB)
powershell -NoProfile -Command "$p=Get-Process vitro; $p.Refresh(); $c1=$p.TotalProcessorTime.TotalSeconds; Start-Sleep -m 8000; $p.Refresh(); $c2=$p.TotalProcessorTime.TotalSeconds; 'CPU {0:P2}  RAM {1:N2}MB' -f (($c2-$c1)/8),($p.WorkingSet64/1MB)"

# 4) 视觉验证:PrintWindow 抓 DComp 内容(CopyFromScreen 抓不到!)
#    完整脚本思路:EnumWindows 找 "Vitro Dock" → PrintWindow(hwnd, memdc, 2)
#    → GetDIBits BGRA → PIL 存 PNG(粘贴到深色底上再放大看)
#    (本会话已在 D:\dockyh\vitro-v3-check.png 等文件验证过此方法)

# 5) 零轮询自证(改了 input.rs/定时器逻辑后必跑)
python vitro/selftest/run-selftest.py   # GetCursorPos/SetTimer/EnumWindows 空闲应 = 0
```

图标再生成(改了 HTML 原型的 SQ 库后):
```bash
python vitro/tools/gen_sq_icons.py docs/prototypes/vitro-nexus-full.html vitro/assets/icons
```

## 6. 环境怪癖(这个开发环境特有的)

- 本 shell 跑在隐藏桌面;**从这里启动的进程会出现在用户的真实桌面**(用户能看到 dock)
- 对这个 shell 截屏只能截到隐藏桌面;验证 dock 渲染用 PrintWindow(§5-4)
- 所有像素坐标是物理像素(进程 PER_MONITOR_AWARE_V2);Python/PowerShell 未声明 DPI 感知时会拿到 1.5x 虚拟化坐标——**测量脚本要先想清楚自己在哪个坐标系**
- GitHub 推送凭据在 Windows 凭据管理器(账号 molniya422),`git push` 直接可用

## 7. 协作偏好(与项目所有者沟通的注意事项)

- 中文交流;**视觉质感是第一诉求**,性能硬预算(RAM≤30MB/空闲≈0%)是不让步的红线
- 明确拒绝过的东西:WebView/Tauri 运行时(内存超标)、轮询式输入、AGPL 代码混入(Glassboard 只可观察行为,不可看代码/复制)
- 用户对"最终版"标准严格——交付前自己先用 PrintWindow 亲眼确认渲染正确,再报告
- 用户上一次不满的直接原因:v2 视觉与 HTML 原型差距大 + 配置损坏丢素材。**任何会动用户配置的改动都要有备份/自愈**
- 里程碑节奏:每个里程碑进 `docs/PLAN.md`(带数字),大决策进 `docs/INTEGRATION.md`

## 8. 当前技术债 / 建议的下一步

1. **exe 9.66MB > 5MB 预算**(boa 8.5MB):可做 feature-gate 把插件系统做成可选编译,或换 WASM 沙箱(wasmtime ~3MB,但 API 迁移工作量大)
2. 玻璃质感与 HTML 原型仍有差距(用户明确指出过):方向是深色 Tahoe 材质调参(theme alpha/tint 与 backdrop_ok 分支的系数)
3. `>` 命令模式无输出捕获(需管道);启动器无 IME 中文输入
4. NeXuS R8 子码头(hover 展开子菜单)是行为规格里最大的未实现项
5. 多显示器:`input.rs` 的 `screen_h`/底边召唤只认主屏;`gpu.rs` 固定画布在副屏 DPI 不同时要重审
6. 发布打包:尚无安装器/自动更新(Glassboard 的 `updates.rs` 思路可参考,注意它 AGPL——思路可学,代码不可抄)
