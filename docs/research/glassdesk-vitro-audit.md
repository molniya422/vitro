# GlassDesk 与 Vitro 审计结论

日期：2026-10-25

这份记录用于新桌面增强套件的架构取舍。它区分“可借鉴的行为和边界”“可以重新实现的算法”和“不能直接进入新产品的代码或资源”。

## GlassDesk（everlosttime/glassdesk-open-source）

GlassDesk 1.4.0 是一个 Windows 11 Dock 与启动台产品，使用 .NET 10 + WPF，并把平台能力集中在手写 Win32/COM 适配层。仓库把职责拆成：

- `GlassDesk.Core`：纯模型、布局、JSON、版本化设置、原子文件和备份；
- `GlassDesk.Platform`：窗口、显示器/DPI、DWM、Shell、托盘、热键、启动和 COM；
- `GlassDesk.AppServices`：应用库、图标缓存、启动/激活、窗口跟踪、搜索和工作区；
- `GlassDesk.Dock`：Dock 窗口、自动隐藏、放大、拖拽重排、Explorer Drop、回收站、菜单和多窗口选择；
- `GlassDesk.Launcher`：Launchpad 分页、搜索、文件夹和拖拽；
- `GlassDesk.Settings` / `GlassDesk.UI` / `GlassDesk.App`：设置、材质、窗口宿主和生命周期。

新产品应吸收这些结构和行为原则：

1. 纯领域层不能依赖 Win32 或渲染器。
2. 平台层必须以真实窗口/应用快照为准，不能根据点击臆造运行状态。
3. 运行窗口跟踪使用事件触发后合并枚举，避免每帧扫描。
4. 拖拽重排先改内存预览，松手才写入；写入失败要回滚。
5. DockLayout 和 LauncherLayout 分开持久化。
6. 透明窗口、DPI、多显示器、托盘、启动和设置都要有独立测试。
7. UI 验收要覆盖真实 Explorer 拖入、重启持久化、窗口激活、自动隐藏和无障碍命中。

不能直接复制 GlassDesk 的代码、品牌、图标或材质资源。该仓库的许可证是范围化授权：新增内容使用 PolyForm Noncommercial 1.0.0，历史 Apache-2.0 部分和第三方组件保持各自条款。新产品采用独立实现，必要时仅保留行为说明和接口思想，并建立自己的来源记录。

## Vitro（本地 `D:\dockyh\vitro`）

Vitro 当前可以通过 Rust 测试：44 个测试通过，但构建有未使用导入、无效 `drop(HANDLE)`、未使用变量和死代码警告。它的价值分成三类：

### 保留为参考

- `src/core/geometry.rs`：Dock 宽度、条目中心、分隔线、底边锚定和子 Dock 几何；
- `src/core/motion.rs`：放大衰减、弹簧步进、子 Dock 开合和动画计时判断；
- `src/core/reducer.rs` / `event.rs` / `model.rs`：悬停、提示、自动隐藏、重排和稳定条目 ID 的纯状态测试；
- `assets/icons`：只作为当前美术资产盘点和重新绘制的参考，不默认作为最终商用资源；
- 性能自测思路和空闲计时器目标。

### 重新实现，不迁移运行时

- `src/dockwin.rs`：窗口、渲染、平台状态和交互耦合过重；
- `src/main.rs` / `src/state.rs`：应用状态和平台调用边界不清；
- `src/lpad.rs`：启动台仍是实验性实现，不能承担正式应用库和文件夹模型；
- `src/config.rs`：路径、默认应用、配置迁移和写入错误处理不足以满足新套件；
- `src/plugin.rs`：Boa 执行任意插件代码，`api.set`/`api.every` 只是演示接口，没有权限、来源、隔离和资源上限；
- `src/runtime.rs`：生命周期改进有价值，但不能直接作为新产品运行时契约；
- `selftest` 和旧固定启动映射：只能作为缺陷记录，不能当验收实现。

### 处理决定

Vitro 保留在仓库中作为 legacy reference，明确不参与新套件编译、发布和运行时依赖。新套件重新实现领域模型、平台适配、插件沙箱、渲染器和持久化；需要算法时先写新测试，再根据行为重新实现。

## 远程仓库

当前本地 Vitro 的 remote 是：

`https://github.com/molniya422/vitro.git`

匿名 Git 读取可以访问该地址，但当前工作环境的 GitHub CLI 和 GitHub 管理连接均未登录，因此不能安全执行“删除仓库”这一远程管理操作。删除分支不等于删除仓库，也不会被当作替代方案。
