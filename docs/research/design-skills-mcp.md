# 调研:热门设计类 Agent Skills 与 MCP 服务器
> 2026-10-24 · 对象:Claude Code/Codex/Cursor 生态中的设计类技能包与设计工具 MCP
> 证据源:Composio 榜单、Paralect MCP 榜单、本地 ui-ux-pro-max-skill 解包实证、Skillry 前文、UX Planet/Medium 评测

---

## 一、设计类 Skills:五种模式 + 热门榜单

### 1.1 五种结构模式(按实现方式分类)
| 模式 | 机制 | 代表 |
|---|---|---|
| **准则类**(注入品味规则) | SKILL.md 文本规则,改变代理的设计决策 | Anthropic frontend-design、Taste |
| **命令集类** | 多个子命令(audit/polish/animate…),按需调用 | Impeccable(23 命令+27 条反模式检测) |
| **搜索数据类**(本地知识库) | 打包可搜索数据库 + 查询 CLI,按需检索而非全文加载 | **UI UX Pro Max**(本地解包实证见 §1.3) |
| **流程类**(工作流编排) | 需求→研究→设计→交接全流程脚本化 | designer-skills(63 技能/27 命令) |
| **验证类**(视觉闭环) | 截图→自评→修复循环 | Playwright/webapp-testing、accesslint(WCAG 审计) |

### 1.2 热门榜单(Composio 2026-05 前十)
1. **frontend-design / frontend-skill**(Anthropic/OpenAI 官方)——27.7 万+ 安装;禁用 Inter/Roboto/Arial/Space Grotesk;强制前期确定美学方向;官方自嘲"禁字体是训练数据问题的创可贴,不是设计哲学"
2. **Impeccable**(Paul Bakaus)——brand/product 双模式(营销页 vs 产品 UI 分治);impeccable.style
3. **figma-implement-design**(OpenAI)——忠实还原 Figma 文件,多状态(hover/empty/loading),配 Playwright 验证
4. **Playwright/webapp-testing**——"对做设计工作的代理,这是单杠杆率最高的东西";截图自评闭环,但"本身不验证品味"
5. **designer-skills**(Owl-Listener)——63 技能覆盖 research→handoff,产出带 QA 清单的交接包;"流程强,品味弱"
6. **theme-factory**——生成可复用 color/spacing/font token,CSS 变量系统,一套源出 light/dark/高对比
7. **designer-skills**(Julian Oczkowski)——requirements-first,需求压测 20+ 分钟提问
8. Composio 集成(Figma/Linear/Notion 连接器,"非工艺类")
9. **excalidraw-diagram-skill**(.excalidraw 可版本控制图表)
10. **accesslint**(WCAG AA/AAA 对比度与语义 HTML 审计)

榜单结论:"**If you only install one, install frontend-design**——这是地板不是天花板";必配 Playwright;**"Taste comes from humans"——技能固化已有品味(约 2x 生产力),不能替你产生品味**。

### 1.3 本地实证:UI UX Pro Max(D:\dockyh\ui-ux-pro-max-skill-main.zip,23MB)
- 结构:**7 个子技能**(banner-design / brand / design / design-system / slides / ui-styling / ui-ux-pro-max),每个 = SKILL.md + references/ + scripts/ + templates/
- 主技能是**可搜索本地设计知识库**:79 风格(50 启用)、192 产品配色+推理档案、74 字体配对、119 条 UX 准则、105 图标、**17 个 GSAP 预设**、25 图表类型、22 技术栈;`python search.py "<query>" --domain ux|style|color|gsap…` CLI 检索
- 亮点设计:**10 级优先级规则表**(Accessibility CRITICAL → Charts LOW),每级带 Must-Have 与 Anti-Patterns;查询契约(最小检索模式/空结果重试一次即声明 fallback,禁止传播未验证输出)
- 这代表第三模式的最高形态:**把设计规范做成"数据库+检索"而非"长文注入"**——按需加载,token 成本可控

### 1.4 评测共识(UX Planet/Medium/react-japan)
- Taste Skill 在视觉质量对比中胜出("most distinctive and polished result")
- Impeccable 对默认 frontend-design 提升有限(react-japan 实测"only slightly better")
- 社区共识:默认 frontend-design"generic",所以第三方技能生态繁荣

---

## 二、设计类 MCP:按工作流六段分类(Paralect 2026)

| 工作流段 | MCP | 核心能力 |
|---|---|---|
| 1 灵感参考 | **Mobbin**(62 万+ 真实 App 截图,付费)· **Awwwards**(获奖站检索+`create_design_brief`) | 真实界面引用,替代"给截图让 AI 猜" |
| 2 可编辑设计文件 | **Paper**(HTML/CSS 画布,帧转 React/Tailwind)· **Penpot**(开源,可自托管,慎用——代理可能误删对象) | 设计文件可读写 |
| 3 组件库 | **shadcn/ui**(注册表安装)· **Storybook**(组件文档+stories)· **21st.dev Magic**(社区精修组件检索/生成)· Vibe(monday.com) | 按真实 API 写代码 |
| 4 资产生成 | **fal**(1000+ 位图/视频模型,按次计费)· **Recraft**(真矢量 SVG 图标/插画/去背) | 图像与图标 |
| 5 反馈与检查 | **Agentation**(点击元素+批注,附带 CSS 选择器/源文件/React 组件树/计算样式,仅本地开发)· **Playwright MCP**(逐断点截图 QA,作者实测自家页面发现 12 个问题) | 设计 QA 闭环 |
| 6 沟通营销 | **Excalidraw**(手绘图表/用户流程)· **Canva**(营销素材多渠道缩放) | 非工程产物 |

**Figma 官方 MCP**:读 frames/变量/Code Connect,但需付费 Dev/Full 席位;免费替代 = **Framelink**(读 Figma 文件)。Glama 注册表 2026-10 已收录 **337 个 Figma 类 MCP**(选 20 强)。

核心观点(原文):"给智能体一张截图它只能猜测间距和颜色;**MCP 让猜测变成真实引用**——真实界面、可编辑的设计文件、有文档的组件。"

### Skills × MCP 的分工(生态级结论)
- **Skills = 注入判断力**(文本规则,改"怎么想");**MCP = 接入真实世界**(工具/数据,改"看到什么")
- 两者在同一工作流互补:灵感(Mobbin MCP)→ 方向(Taste skill)→ 实现(+组件 MCP)→ 验证(Playwright)——即 Composio 推荐的最小闭环

---

## 三、对 Vitro 的落点

1. **我们的资产已在赛道上**:transitions-dev skill(动效令牌)、rea-prompts(逆向工作流)与上述技能同构;ui-ux-pro-max 的"搜索式知识库"模式值得抄给**插件开发者文档**(119 条 UX 准则 → 检索式,而非长文)
2. **验证闭环已自研等效**:Composio 把 Playwright 截图自评列为"最高杠杆";Vitro 已有 PrintWindow 抓图 + selftest 零轮询自证 + 性能门——**同样的闭环思想,原生领域自己造**(通用 Playwright 抓不到 DComp)
3. **可新增一个"设计审查"技能**(借鉴 Impeccable):vitro-design-review —— audit(对照 macos-parity 令牌检查间距/对比度)/ polish(按效果目录逐槽检查)/ animate(检查 reduced-motion 降级);产物进 docs
4. **图标遮罩/统一风格**可借鉴 Recraft 的"真矢量+统一风格"思路:SQ 图标管线已保证一致性,风格混杂问题用 §4.4 遮罩方案(已在 backlog)
5. **远期想法(记录不实施)**:Vitro Dock MCP —— 让 agent 能查询 dock 状态/装插件/改主题(用户说"帮我把 dock 改成暗色+加个 CPU 表"直接生效);依赖插件系统稳定后(v4+)

## 附:来源
[Composio:10 best design skills](https://composio.dev/content/top-design-skills) · [Paralect:Best design MCP servers 2026](https://www.paralect.com/mcp/top-design-mcp-servers) · [UX Planet 三技能实测](https://uxplanet.org/i-tested-3-claude-code-skills-for-web-design-which-one-is-best-521137220dc1) · [Medium:5 skills that fix ugly UI](https://divadsanders.medium.com/5-claude-skills-that-fix-claudes-ugly-ui-ai-web-design-ec9cd27bf5fe) · [react-japan 实测](https://react-japan.dev/articles/three-design-tools-one-model) · [Anthropic frontend-design SKILL.md](https://github.com/anthropics/claude-code/blob/main/plugins/frontend-design/skills/frontend-design/SKILL.md) · [Taste Skill](https://www.tasteskill.dev) · [taste-skill GitHub](https://github.com/leonxlnx/taste-skill) · [21st.dev Magic MCP](https://mcpservers.org/servers/21st-dev/magic-mcp) · [Glama Figma MCP 榜](https://glama.ai/mcp/best/figma) · [Builder.io MCP 2026](https://www.builder.io/blog/best-mcp-servers-2026) · 本地解包:ui-ux-pro-max-skill-main(23MB,7 技能)
