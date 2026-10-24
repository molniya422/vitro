# 调研:Skillry(skillry.dev)的高级感设计思路
> 2026-10-24 · 对象:AI Agent Skill 市场 "Agent Skills with taste"($9.99/月,400+ 技能,2026-08 上线,有[中文版](https://skillry.dev/zh))
> 姊妹篇:frontend-design-research.md / design-language-research.md

---

## 0. 它是什么
给 Claude Code / Codex / Cursor 等编码代理装的**技能包市场**——每个 Skill 是一份 SKILL.md 工作流(指令+模板+脚本),让代理产出网页/幻灯片/视频/图片。核心卖点不是功能是**品味**;中文社区口碑:"taste 的、执行得很好的,目前看到最强的就是 yihui 的这个 skillry"。

## 1. 高级感的第一来源:**品味即产品(curation as the product)**
- 口号级判断:"**curation really is the product**""curation beats complete"——高级感来自**人工策展 + 统一品质标准**,不是功能堆叠
- 生产流程四步,设计先行:"**01 Design the result**(先由美术指导做出完成品)→ 02 写工作流 → 03 真实跑出成果 → 04 持续维护"
- "One standard across every Skill":全库一个标准;技能页**先看真实产出再买**,对比式展示(参考物 vs 代理复刻 + 完整 prompt 并排)——**用证据建立高级感**
- 站点自身:粉彩小图标被用户点名"比普通目录 premium 得多"——**小而精的细节抬升整体档次**

## 2. 第二来源:**反 AI 味(anti-slop)——高级 = 不像生成的**
明确列出的 AI 套路禁项(原文):
> centred layouts(居中布局)、default fonts(默认字体)、pale blue buttons(淡蓝按钮)、bland hover states(平庸悬停态)

其推荐的 **Taste** 技能(独立开源项目)方法论:
1. 读简报 → 推断设计方向
2. 设三个拨盘:**variance(变化度)/ motion(动效)/ density(密度)**
3. 映射到真实设计系统(非模板)
4. 出货前跑 **AI UI 模式 ban list**

## 3. 第三来源:**先承诺方向,再写代码**
写码前必须先定四要素(原文):"a **typographic voice**, a **colour story**, a **layout idea** and a **motion style**"(排版的声调、色彩的故事、布局的构思、动效的风格)。模糊简报是最常见失败原因。

## 4. 第四来源:**旗舰技能的"融资级"设计语言**(以 Cadence Marketing Landing 为例,$4.99,327 装机,5.0 分)
- **配色**:炭黑底色 + **单一青色强调**("a charcoal ground, one cyan accent")——单强调色纪律
- **排版**:灰→白渐变大标题;**Geist / Geist Mono** 字体内嵌离线可用
- **动效**:滚动驱动的"可信产品瞬间"——"laptop mockup whose lid opens on scroll"(滚动时笔记本开盖)、产品屏随滚动立起
- **结构**:单页**一条连续产品叙事**(英雄区→跑马灯→特性→工作流→客户→定价月/年切换→FAQ→注册);"looks funded on day one"(第一天就显融资级)是它的原话卖点
- 工程侧:单文件自包含、离线可开;标签 Style-locked(风格锁定)+ Conversion-focused

## 5. 方法论分层:准则类 × 模板类互补
| 类型 | 代表 | 作用 | 局限 |
|---|---|---|---|
| 准则类 | Anthropic frontend-design、**Taste**、Impeccable(24 命令:audit/critique/polish/animate;维度:排版/对比/空间/动效/响应式/UX 文案) | 改变代理的**设计思路** | 结果随模型与简报波动 |
| 模板类 | Skillry 站内技能("技能页上的布局就是你得到的布局") | **可预测的成品起点** | 与品牌契合度需调 |

## 6. 对 Vitro 的直接应用
1. **四要素前置承诺** → 主题引擎每套主题先定:排版声/色彩故事/布局想法/动效风格,再落 TOML(主题文件正好承载四要素)
2. **反 AI 味自检清单** → 原型/官网设计禁项:居中布局、默认字体、淡蓝按钮、平庸 hover——设为 UI review 的 ban list
3. **三拨盘参数化** → 我们已有 motion 拨盘(动效档案);主题引擎补 **density(信息密度)** 与 **variance(视觉变化度)** 两个全局维度,用户三滑杆调"质感"
4. **深底+单强调色+渐变标题+滚动微叙事** → Vitro 官网/设置页设计语言直接套用("looks funded on day one")
5. **先看结果再买** → 复刻配置码分享时强制带预览截图/在线预览链接(信任建立)
6. **策展思维** → 本地插件目录加 精选/评分/装机数徽标(我们已有目录结构,补策展字段);官方首发插件按"一个标准"打磨

## 附:来源
[Skillry 首页](https://skillry.dev/) · [中文版](https://skillry.dev/zh) · [Best frontend design Skills(方法论正文)](https://skillry.dev/learn/frontend-design-skills) · [Cadence 技能页](https://skillry.dev/skills/bs-cadence-marketing-landing) · [博客发刊词](https://skillry.dev/blog/introducing-skillry) · [Taste Skill 官网](https://www.tasteskill.dev) · [taste-skill GitHub](https://github.com/leonxlnx/taste-skill) · [AIidelist Review](https://aiidelist.com/blog/skillry-review) · [Reddit 讨论](https://www.reddit.com/r/claudeskills/comments/1w2imnh/i_built_a_skill_to_give_claude_some_design_taste)
