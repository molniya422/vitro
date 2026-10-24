# Vitro Dock

原生 Windows 的 macOS 风格 Dock —— Rust + DirectComposition,零轮询事件驱动。

![screenshot](docs/screenshot.png)

**硬预算**:RAM ≤ 30MB(实测 ~7MB)· 空闲 CPU ≈ 0%(实测 0.3–0.6%)· 单文件原生 exe,无 WebView。

## 是什么

Vitro 是一个 Windows 桌面 Dock:

- **玻璃药丸**:DWM 系统亚克力(Win11 22H2+,旧系统自动回退)+ 1px 描边 + 内高光/内影,Tahoe 材质参数
- **falloff 弹簧放大**:鼠标邻近图标按衰减曲线放大,松手弹簧回落(峰值/半径可在主题里调)
- **真实图标**:应用磁贴用 Shell 三档提取(IShellItemImageFactory@96 → SHGetFileInfoW@32 → 内置图标兜底)
- **SQ 图标库**:25 枚内置图标(时钟/天气/CPU/电池/网速…)从设计原型栅格化内嵌,插件磁贴 = 图标 + 数值徽章
- **倒影**:PushLayer + 渐变遮罩镜像,随弹簧缩放联动
- **呼吸指示**:运行中程序的指示点呼吸频率 = 该进程实时 CPU 占用(创新点)
- **启动器**:Ctrl+Alt+D 全屏抽屉(搜索 / `>` 命令 / `=` 计算三模式)
- **插件**:boa_engine(纯 Rust JS 引擎)沙箱,记事本写代码存进 TOML 即装
- **零轮询**:WH_MOUSE_LL 钩子 → 邻域门控 → 事件驱动;光标离开 ±200px 邻域 = 0 消息 0 渲染 0 定时器
- **底边召唤**(可选):自动隐藏后鼠标移到屏幕底边 3px 唤出,全程消息驱动
- **主题热重载**:记事本改 `theme.toml`,Ctrl+Alt+R 生效

## 性能

| 指标 | 预算 | 实测 | 对照 |
|---|---|---|---|
| RAM(工作集) | ≤30MB | **7.09MB** | WebView 方案 70–120MB |
| 空闲 CPU(8s 采样) | ≈0% | **0.59%** | 同类轮询架构 8–17% |
| 空闲系统调用 | 0 | GetCursorPos/SetTimer/EnumWindows = **0 次/10s**(Frida 自证) | — |

自证方法与证据:`vitro/selftest/`(用逆向 NeXuS 的同一套 Frida 钩子对拍自己)。

## 构建

```
cargo build --release        # 在 vitro/ 下
./target/release/vitro.exe
```

依赖:Rust stable、Windows 10/11。无其他运行时。

## 配置

| 文件 | 作用 |
|---|---|
| `~/.vitro/dock.toml` | 应用列表([[items]])、插件([[plugins]])、autohide |
| `~/.vitro/theme.toml` | 玻璃 alpha/tint、accent、放大 peak/radius |

操作:左键启动 · 右键菜单(移除/启动器/主题/退出) · 拖放文件加磁贴 · Ctrl+Alt+D 启动器 · Ctrl+Alt+R 重载主题 · 托盘图标管自动隐藏。

写一个插件:

```toml
[[plugins]]
name = "时钟"
code = "api.set('12:34')"
# icon = "clock"        # 可选:SQ 图标键,缺省按名称猜
```

## 架构

```
vitro/src/
├── main.rs     入口/装配(固定 1024×124 透明画布,永不 Resize)
├── gpu.rs      D3D11→D2D→DComp 管线一次成型(布局自适应全在绘制层)
├── state.rs    唯一状态 + 药丸几何(局部/屏幕两套坐标)
├── dockwin.rs  窗口过程 + 渲染(玻璃/放大/呼吸/倒影/徽章)+ 交互
├── input.rs    WH_MOUSE_LL 零轮询输入 + R9 底边召唤
├── config.rs   TOML 自愈读写 + Tahoe 主题
├── icons.rs    Shell 三档图标 + SQ 库(png 解码→预乘 BGRA→D2D)
├── tray.rs     托盘
├── lpad.rs     启动器抽屉
└── plugin.rs   boa JS 沙箱(api.set/every/log)
```

关键设计:窗口是**固定尺寸透明画布**,玻璃药丸按磁贴数在绘制层自适应宽度居中,药丸外像素由 DComp 透明点击穿透 —— 不存在运行期交换链 resize 这一 bug 类。

## 文档

| 文档 | 内容 |
|---|---|
| [HANDOVER.md](HANDOVER.md) | ★ 接手文档:上手清单/架构决策/踩坑/验证手段/协作偏好 |
| [docs/INTEGRATION.md](docs/INTEGRATION.md) | 资源整合权威表(NeXuS/Glassboard/macOS 规格/HTML 原型四方取舍) |
| [docs/PLAN.md](docs/PLAN.md) | M0–M7 里程碑日志(含每步性能数字与技术笔记) |
| [docs/research/](docs/research/) | NeXuS 行为逆向规格(Frida 证据)、macOS Dock 设计规格、竞品调研 |
| [docs/specs/](docs/specs/) | 动效令牌、效果目录、macOS 对拍参数 |
| [docs/prototypes/](docs/prototypes/) | 交互式 HTML 设计原型(浏览器直接打开,视觉验收基准) |
| [vitro/tools/gen_sq_icons.py](vitro/tools/gen_sq_icons.py) | SQ 图标库再生成管线 |

## 许可与致谢

- 本仓库代码:MIT(见 LICENSE)
- 行为参考:对 Winstep NeXuS 的**独立行为观察与文档**(未复制其任何代码/资源)
- 结构参考:[Glassboard](https://github.com/ImNallen/Glassboard)(AGPL-3.0,仅学习其 Win32 模块化组织方式,未引入代码,故不入库)
- 动效参数:transitions.dev 片段库(参数级参考)
- SQ 图标库为本项目自绘 SVG 栅格化产物
