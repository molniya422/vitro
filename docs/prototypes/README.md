# 交互式设计原型

这 4 个自包含 HTML 文件是 Vitro 的**视觉/交互验收基准**,浏览器直接打开即可:

| 文件 | 内容 |
|---|---|
| `vitro-nexus-full.html` | 全功能原型(NeXuS 行为对拍 + SQ 图标库 + 复刻中心),**主交付物** |
| `vitro-macos-parity.html` | macOS Sonoma/Tahoe 材质对照(Tahoe 玻璃参数出处) |
| `vitro-m05.html` | 抽屉启动器 + 命令面板交互原型 |
| `vitro-effects-gallery.html` | 可选动效画廊(用户可开关的效果清单) |

SQ 图标库再生成:`vitro/tools/gen_sq_icons.py`(从这些 HTML 提取 SVG → Chromium 栅格化 → 原生内嵌)。
