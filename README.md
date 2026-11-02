# Vitro Dock

一个 Windows 底部 Dock，使用 **Tauri 2 + React** 构建，系统指标由一个 **Python
`psutil` sidecar** 通过 stdio JSON-RPC 提供。

产品代码位于 [`dock-tauri/`](dock-tauri/)。当前 `main` 只包含产品树（代码、文档、美术源）；
早期原型（`vitro/`、`dock-app/`）不在此树中，保留在
`backup/main-pre-reshape`、`backup/main-local` 和 `codex/vitro-appruntime` 分支里。

## 功能

- **Dock**：真实 Start Menu 图标（按 DPI 提取）、可调放大/倒影/提示、运行点/最小化角标、
  右键菜单、拖动重排、启动脉冲、点击运行中应用聚焦/还原。
- **点击穿透**：透明 headroom 与反射带不拦截点击；拖动时临时接管整窗。
- **自动隐藏**：按 pill 判定，延迟可调；全屏应用前台时让位。
- **Launchpad**：全量应用搜索；拖到投放条可插入 Dock 的指定位置。
- **Shelf**：拖文件到窗口即可暂存（会话级、最多 12 项），点击打开、删除、清空。
- **系统监控**：CPU / 内存 / 磁盘 / 网络；sidecar 启动预热，45 s 无采样自动退出，
  打开 monitor 时重新预热。
- **设置**：外观（尺寸/间距/内边距/圆角/颜色/透明度/倒影）、行为、自动隐藏、入口显隐、
  应用增删排序，实时预览并原子持久化（schema 3，`.bak` 恢复）。
- **性能**（release，先展开一次再收起）：5.9 MB host / 186.6 MB 全树 / 0.03% 空闲 CPU。

## 目录

```
dock-tauri/             产品：Tauri 宿主、React 前端、Python sidecar、E2E、脚本
dock-tauri/design-art/  美术资源包（图标、字形、主题、3D 样张）
docs/                   设计规范、阶段计划、研究资料、历史文档
```

## 快速开始

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"
cd dock-tauri
npm install
npm run tauri dev
```

要求 Windows 11 + WebView2、Rust 1.89+、Node 20+、Python 3.11 + `psutil`。
详细说明见 [`dock-tauri/README.md`](dock-tauri/README.md)。

## 测试

```bash
cd dock-tauri
npm test                 # Vitest（80）
npm run test:e2e         # Playwright（20，mock Tauri IPC）
cargo test --manifest-path src-tauri/Cargo.toml   # Rust（132）
cd backend && python -m pytest tests -q           # Python（9）
```

## 文档

- 产品说明：[`dock-tauri/README.md`](dock-tauri/README.md)
- 接手文档（中文，当前状态/调试/约束）：[`dock-tauri/HANDOVER.md`](dock-tauri/HANDOVER.md)
- 设计规范：[`docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`](docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md)
- 阶段计划：[Phase 1](docs/superpowers/plans/2026-10-26-tauri-python-dock-implementation.md) ·
  [Phase 2](docs/superpowers/plans/2026-10-26-dock-tauri-phase2.md) ·
  [Phase 3](docs/superpowers/plans/2026-10-26-dock-tauri-phase3.md) ·
  [Phase 4](docs/superpowers/plans/2026-10-28-dock-tauri-phase4-drag.md) ·
  [Phase 5](docs/superpowers/plans/2026-10-29-dock-tauri-phase5-shelf.md)
- 历史文档：`docs/legacy/`、`docs/research/`、`docs/prototypes/`

## 发布

sidecar 冻结与 MSI 构建步骤见 [`dock-tauri/README.md`](dock-tauri/README.md)。
当前 MSI 已构建（9.48 MiB），未安装。

## 许可

见 [`LICENSE`](LICENSE)。`dock-tauri/design-art/vendor/` 中的 Three.js 自带 MIT 许可。
