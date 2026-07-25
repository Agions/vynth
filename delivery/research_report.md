# AICoding 架构设计 · 行业调研报告

> 本文档为《AICoding 架构设计》核心产物之一，定位为**行业调研报告（research_report）**。
> 上游输入：主理人（team-lead）转交的用户诉求 + 调研目标；
> 下游输出：驱动 `business-architect`（业务架构师）的行业调研判断，最终落入《高层架构设计》的 §3 行业调研章节。
> 产出者：`research-analyst`（研究分析师 - 查有据）。本文档为**建议而非裁决**——最终业务边界由下游业务架构师冻结。

---

## 0. 元信息：修订记录

```yaml
标题: Vynth（vibe coding TUI 终端编程工具） - 行业调研报告 v0.1
版本: v0.1
状态: Draft   # Draft | Reviewing | Approved | Deprecated
创建日期: 2026-07-24
最后更新: 2026-07-24
调研人: research-analyst（查有据）
审核人:
  - team-lead（主理人）

关联文档:
  上游输入:
    - 用户诉求: 由主理人注入（vibe coding TUI 终端编程工具，类比 Claude Code/OpenCode/Codex）
    - 调研目标: 由主理人注入（6 大调研方向）
    - 项目资料摘要: material_digest.md v0.1（knowledge-ingest-engine 产出，G1 通过）
  下游产出:
    - 高层架构设计 §3 行业调研: 将由 business-architect 整合到此章节
```

| 版本 | 日期 | 作者 | 变更内容 | 评审状态 |
| --- | --- | --- | --- | --- |
| v0.1 | 2026-07-24 | `research-analyst` | 初稿（G2 阶段，对标 5 家竞品 + Vynth 自研对照） | Draft |

---

## 1. 调研问题收敛

> 围绕主理人注入的用户诉求与 6 大调研方向，先收敛为明确的调研问题集合，确保调研不偏离当前项目背景。

### 1.1 原始调研种子

| 编号 | 待验证论题 | 来源（用户诉求 / 调研目标要点） | 调研优先级 | 备注 |
| --- | --- | --- | --- | --- |
| S1 | 竞品（Claude Code/OpenCode/Codex/Aider/Gemini CLI）的终端形态、agent loop、工具集、上下文管理与流式渲染范式 | 调研目标 ① 竞品对标 | 高 | 确定 Vynth 应借鉴的架构范式 |
| S2 | 终端 TUI 渲染 ink vs 自研 ANSI 的取舍；流式输出不阻塞渲染的逃生舱模式 | 调研目标 ② 终端 TUI 渲染技术 | 高 | Vynth 已选非 ink，需验证合理性 |
| S3 | 单二进制分发（Bun / Node esbuild / Rust）体积、冷启动、生态约束 | 调研目标 ③ 单二进制分发 | 高 | Vynth 当前 61MB、目标 20–40MB |
| S4 | Agent 循环与工具系统（tool call 聚合、maxSteps、sandbox 隔离强度） | 调研目标 ④ Agent 循环与工具系统 | 高 | Vynth run_shell 当前无隔离（显著风险） |
| S5 | MCP stdio JSON-RPC 客户端接入形态、插件生命周期 | 调研目标 ⑤ MCP 与插件扩展 | 中 | Vynth mcp 未并入 CLI、plugins 已接入无头 |
| S6 | 终端编程工具的信任/安全模型（宿主权限 vs 硬隔离） | 调研目标 ⑥ 安全/信任模型 | 高 | Vynth 当前为设计信任、无硬隔离 |

### 1.2 调研问题收敛

| 编号 | 调研问题 | 调研对象 | 调研目标 | 预期产出 | 关联种子 |
| --- | --- | --- | --- | --- | --- |
| Q1 | 主流终端编程 Agent 的终端形态、agent loop、工具集、上下文管理与流式渲染范式是什么？ | Claude Code / OpenCode / Aider / Codex CLI / Gemini CLI 官方文档与架构拆解 | 提炼可借鉴的 agent-loop / 工具系统 / plan-mode / 流式渲染范式 | 竞品架构对比表（§2.1–§2.3） | S1 |
| Q2 | 终端 TUI 渲染「ink vs 自研 ANSI」的取舍与流式不阻塞渲染的最佳实践是什么？ | Ink 渲染管线、Claude Code Ink 实现、Vynth 自研 ANSI + stream-escape-hatch（D28/D29） | 确认 Vynth 非 ink 选型的合理性及逃生舱模式最佳实践 | TUI 渲染技术对比 + 建议（§4.3） | S2 |
| Q3 | 单二进制分发（Bun vs Node SEA vs Deno vs Rust）的体积 / 冷启动 / 生态约束如何？ | Bun compile 文档与基准、Claude Code Native 100MB、Node SEA 基准、Vynth 当前体积（D52/D53/D54） | 定位 Vynth 61MB / 目标 20–40MB 的行业位置与优化路径 | 单二进制技术对比 + 建议（§4.3） | S3 |
| Q4 | Agent 循环与工具系统（tool call 聚合、maxSteps、sandbox 隔离强度）的行业实践如何？ | 各竞品 agent-loop、sandbox 隔离（Claude Code bubblewrap/seatbelt、Codex Seatbelt/Docker、Gemini CLI 多层沙箱）、Vynth sandbox（D34） | 评估 Vynth maxSteps=8、run_shell 无隔离的风险与改进方向 | agent / sandbox 对比 + 风险（§5.1 R-01） | S4 |
| Q5 | MCP stdio JSON-RPC 与插件扩展的接入形态如何？ | MCP 规范（stdio 传输）、Claude Code / Gemini CLI MCP-first、Vynth McpClient（D38）与 plugins loader（D42） | 确定 Vynth MCP 未接入 CLI 的差距与接入路径 | MCP / 插件对比 + 建议（§4.1） | S5 |
| Q6 | 终端编程工具的信任/安全模型（宿主权限 vs 硬隔离）如何取舍？ | Claude Code 沙箱、Codex 安全模型、Gemini CLI 沙箱、AI coding agent 威胁模型、Vynth 信任边界（D53/D54） | 评估 Vynth 设计信任模型的风险与 OS 级硬隔离的必要性 | 安全模型对比 + 风险（§5.1 R-01） | S6 |

---

## 2. 事实：标杆系统盘点和方案详述

> **四段式「事实」段**。只陈列调研发现的事实，不做引申建议或边界裁决。竞品事实标注来源 URL/章节；Vynth 自身事实标注 `material_digest` 的 D 编号出处。

### 2.1 行业标杆清单

**硬指标**：≥ 3 家；至少包含 1 家头部 SaaS 代表（Claude Code / Codex CLI / Gemini CLI）+ 1 家开源/自研代表（OpenCode / Aider / Vynth 自研）。

| 编号 | 标杆系统 | 厂商 / 社区 | 部署形态 | 场景覆盖 | 技术亮点 | 商业模式 | 调研来源 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B1 | Claude Code | Anthropic（闭源 SaaS + 本地 CLI 客户端） | 本地 CLI 客户端 + 云端模型（Anthropic 服务器） | 终端自主编程 Agent、IDE/Web 扩展 | agentic loop（gather→act→verify）、6 级权限、CLAUDE.md、MCP/hooks/subagents、上下文压缩 | 订阅制（Claude.ai / API 用量） | SR-01 / SR-02 / SR-03 |
| B2 | OpenCode | Anomaly 社区（开源 MIT） | 本地优先，单二进制/多端（CLI/Desktop/Web/IDE） | 模型中立的终端 AI 编程 Agent | TypeScript + Effect.ts、Provider 无关、Plan/Build 双模式、AGENTS.md/Skills/MCP、LSP | 免费开源（MIT），自备 API Key | SR-05 / SR-06 / SR-07 |
| B3 | Aider | Paul Gauthier（开源 Apache 2.0） | 本地 CLI + Git 仓库 | 终端结对编程（git-diff 编辑 + 自动 commit） | repo-map（tree-sitter AST）、多种 edit format、Architect Mode 双模型、Git 审计轨迹 | 免费开源（Apache 2.0），仅付 API token | SR-08 / SR-09 |
| B4 | Codex CLI | OpenAI（开源 Apache 2.0 + 云端模型） | 本地 CLI + 云端模型（OpenAI） | 终端轻量 coding agent、沙箱执行 | 三档审批模式、apply_patch、Seatbelt/Docker 沙箱、AGENTS.md、MCP | 免费开源，付 OpenAI API | SR-10 / SR-11 / SR-12 / SR-13 |
| B5 | Gemini CLI | Google（开源 Apache 2.0） | 本地 CLI + 云端模型（Gemini） | 终端 AI Agent、1M 上下文、MCP-first | 1M token 上下文、原生 MCP、多层沙箱、Headless JSON、GEMINI.md | 免费 OAuth 额度 + 付费 API | SR-14 / SR-15 |
| B6 | **Vynth（本项目，对照基准）** | 自研（vynth 团队） | 本地优先单二进制（Bun compile） | vibe coding TUI 终端编程工具 | 自研轻量 ANSI + 逃生舱、OpenAI 兼容 SSE、Plan/Vibe 双模式、插件无头接入；MCP 未并入 CLI | 自研，默认 DeepSeek（D14） | D1 / D14 / D24 / D28 / D34 / D38 / D52 |

### 2.2 标杆方案详述

> 每家标杆逐一展开（B1–B5 详述 + B6 Vynth 对照）；每行区分「已核实的事实」与「推断/假设」，标注置信度。

#### 2.2.1 B1 - Claude Code（Anthropic）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | 开发者结对编程伙伴 / 自主 coding agent，运行于终端，可自主读项目、执行命令、改多文件、跑测试、Git 操作 | 已核实（SR-02 / SR-03） |
| 目标用户 | 专业开发者、工程团队、企业（含合规部署） | 已核实（SR-03） |
| 核心能力 | agentic loop（gather context→act→verify→repeat，单线程避免多 agent 复杂度）；5 类内置工具（文件读写/编辑/命令执行/网络研究/外部服务）；Plan Mode（先规划后执行，多文件任务架构错误降约 45%）；CLAUDE.md 项目记忆 | 已核实（SR-01 / SR-03） |
| 架构特点 | 本地 CLI 客户端 + 云端模型（"浏览器与网站"关系）；SSE/tool_calls 聚合；上下文约 92% 容量时自动 compaction；MCP/hooks/subagents 扩展；6 级权限（只读→编辑→受限命令→全命令→Git→管理员） | 已核实（SR-02）；subagents 为推断（SR-01） |
| 部署形态 | 本地 CLI（npm 安装 / 原生安装器 beta）；云端模型经 Anthropic API；支持私有云 / 本地部署 / 混合；"Claude Code on the web" 云端隔离沙箱 | 已核实（SR-03） |
| 集成方式 | MCP（Model Context Protocol）、hooks、subagents、IDE 扩展（VS Code / JetBrains）、Web | 已核实（SR-01 / SR-03） |
| 定价模式 | 订阅制（Claude.ai 账户 / API 用量计费）；企业私有云/本地部署 | 已核实（SR-03） |
| 优势 | 工程成熟度最高（GA 后 $1B+ ARR，2025-11）、SWE-bench Verified 77.2%（并行 82%）、生态与文档完善、流式 TUI 用 Ink（React）渲染（389 个 UI 文件，虚拟列表 O(visible)） | 综合归纳（SR-03 / SR-20） |
| 局限 | 闭源、云模型依赖（数据出本地除非私有云）；权限弹窗疲劳（沙箱后降 84%）；单二进制 native build 约 100MB（bun compile，含 Bun 运行时 ~95MB） | 已核实（SR-04 / SR-21）；100MB 为推断（SR-04 社区报道） |
| 对本项目的参考价值 | agent-loop / Plan 模式 / 6 级权限 / 上下文压缩 / 流式 TUI 范式高度可借鉴；其沙箱（bubblewrap/seatbelt，见 §2.2.1 架构特点扩展）是 Vynth 无隔离现状的对标改进方向 | 推断 |

> 沙箱补充（置信度 已核实，SR-21）：Claude Code 新增沙箱（beta），基于 OS 级原语（Linux bubblewrap / macOS seatbelt），同时做文件系统隔离 + 网络隔离，沙箱化后权限弹窗降 84%；已开源该研究预览。

#### 2.2.2 B2 - OpenCode（Anomaly 社区，开源 MIT）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | 开源、模型中立的终端 AI 编程 Agent，"开源版 Claude Code"，可接任意模型 | 已核实（SR-05 / SR-06） |
| 目标用户 | 终端开发者、隐私敏感环境、避免供应商锁定者 | 已核实（SR-05） |
| 核心能力 | Provider 无关（75+ LLM，含本地模型）；Plan（只读）/Build（全权限）双模式；AGENTS.md/Skills/MCP 三类"记忆"；LSP 集成；PTY 管理；SQLite 会话持久化 | 已核实（SR-06 / SR-07） |
| 架构特点 | TypeScript 全栈 + Effect.ts（函数式效果系统，fiber 并发/依赖注入）；Vercel AI SDK 统一 75+ 模型；Turborepo monorepo；客户端/服务端分离（Hono HTTP/WebSocket）；Bun 可选运行时；yargs CLI | 已核实（SR-06 / SR-07）；Effect.ts 学习曲线为推断（SR-06） |
| 部署形态 | 本地优先，单二进制/多端（CLI / Electron Desktop / SolidJS Web / VS Code 扩展 / SDK）；隐私优先（不存储用户代码/上下文） | 已核实（SR-05 / SR-07） |
| 集成方式 | MCP Server、Skills 插件、AGENTS.md、LSP、多端 SDK/API | 已核实（SR-07） |
| 定价模式 | 免费开源（MIT），用户自备 API Key，无许可费 | 已核实（SR-05） |
| 优势 | 开源可审计、模型中立无锁定、隐私优先、社区活跃（≥160K stars / 900+ 贡献者 / 月活 7.5M，SR-05）、与 Vynth 同为 TS + 本地优先 + Plan/Build 双模式，范式最贴近 | 综合归纳（SR-05 / SR-06） |
| 局限 | 架构复杂度高（Effect.ts 陡峭学习曲线）；仓库地址/归档状态社区存在争议（见 §5.2 U-01）；沙箱隔离细节公开资料较少 | 推断（仓库争议待确认） |
| 对本项目的参考价值 | 本地优先 + 开源 + 模型中立 + Plan/Build 双模式 + Provider 抽象层，与 Vynth 定位最契合，是优先借鉴对象；其目录级权限/记忆机制可参考 | 推断 |

#### 2.2.3 B3 - Aider（Paul Gauthier，开源 Apache 2.0）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | 终端结对编程工具，将 LLM 与本地 Git 仓库配对，以自然语言产出多文件 diff 并自动提交 | 已核实（SR-08 / SR-09） |
| 目标用户 | 终端开发者、Git 工作流偏好者、需可审计变更轨迹者 | 已核实（SR-09） |
| 核心能力 | chat loop + repository map（tree-sitter AST 大纲，按图重要性打分装入 token 预算）+ edit applier；多种 edit format（whole/diff/udiff/editor）；Architect Mode 双模型；每轮编辑自动 git commit（/undo 回滚） | 已核实（SR-08 / SR-09） |
| 架构特点 | Python 实现（与多数 TS 竞品不同）；Coder 多态层级（EditBlock/WholeFile/UnifiedDiff）；repo-map 基于 tree-sitter，跨 100+ 语言；上下文压缩（history.summarize） | 已核实（SR-08 / SR-09） |
| 部署形态 | 本地 CLI（pip 安装），模型经远端 API；可选浏览器 UI | 已核实（SR-09） |
| 集成方式 | 模型无关（Claude/GPT/Gemini/DeepSeek/Grok/Ollama）；/add、/undo、/diff 命令；CI 友好 | 已核实（SR-09） |
| 定价模式 | 免费开源（Apache 2.0），仅付 API token（多数开发者 $30–60/月，SR-09） | 已核实（SR-09） |
| 优势 | 最成熟（2023 起），repo-map 上下文工程标杆，Architect Mode SWE-bench 85/100（SR-09），Git 审计轨迹天然防错 | 综合归纳（SR-09） |
| 局限 | Python 技术栈与 Vynth TS 不直接兼容；非通用 agent-loop+tool 范式（偏 git-diff 编辑）；沙箱依赖用户/Git 而非 OS 级隔离 | 推断 |
| 对本项目的参考价值 | repo-map 上下文压缩、edit format 选择、Git 审计轨迹思路可借鉴；其"模型无关 + 本地"理念与 Vynth 一致 | 推断 |

#### 2.2.4 B4 - Codex CLI（OpenAI，开源 Apache 2.0）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | OpenAI 官方终端轻量 coding agent，本地运行、云端模型，可沙箱内自主执行 | 已核实（SR-10 / SR-13） |
| 目标用户 | 终端开发者、OpenAI 模型用户、需沙箱安全执行者 | 已核实（SR-10） |
| 核心能力 | 三档审批模式（Suggest 默认 / Auto Edit / Full Auto）；apply_patch（结构化 diff 原语）；AGENTS.md；MCP（CLI + IDE 共享）；/plan、/resume | 已核实（SR-10 / SR-12 / SR-22） |
| 架构特点 | 本地 CLI（npm install -g @openai/codex，Node.js 22+）；ReAct 式 tool-use loop 运行于沙箱内；审批策略与沙箱模式解耦（untrusted/on-request/never） | 已核实（SR-10 / SR-13 / SR-22）；"语言 Rust+TypeScript" 为第三方目录说法（SR-22），与官方 npm/Node 文档冲突，标记为待核实/降级（见 §5.2 U-01 说明） |
| 部署形态 | 本地 CLI + 云端 OpenAI 模型；Full Auto 默认网络禁用 + 目录沙箱 | 已核实（SR-10 / SR-13） |
| 集成方式 | MCP（first-class，CLI 与 IDE 共享配置）、AGENTS.md、GitHub 集成 | 已核实（SR-22） |
| 定价模式 | 免费开源（Apache 2.0），付 OpenAI API（默认 o4-mini / o3） | 已核实（SR-10 / SR-13） |
| 优势 | 开源可审计、审批模式分级清晰、沙箱成熟（macOS Seatbelt 只读 jail / Linux Docker + iptables 仅放行 OpenAI API）、OpenTelemetry 审计日志 | 综合归纳（SR-11 / SR-13） |
| 局限 | OpenAI 模型锁定（无多 provider）；Windows 沙箱需 Docker；生态较新 | 已核实（SR-13） |
| 对本项目的参考价值 | 审批模式分级、apply_patch、Seatbelt/Docker 沙箱、OpenTelemetry 审计日志是可借鉴的安全/可控设计；OpenAI 锁定与 Vynth 多 provider 定位不符 | 推断 |

#### 2.2.5 B5 - Gemini CLI（Google，开源 Apache 2.0）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | Google 官方开源终端 AI Agent，将 Gemini 直接带入终端，1M 上下文 + MCP-first | 已核实（SR-14 / SR-15） |
| 目标用户 | 终端开发者、大代码库、Google Cloud 生态、成本敏感者（免费 OAuth 额度） | 已核实（SR-15） |
| 核心能力 | 1M token 上下文（整库分析）；原生 MCP（first-class）；内置 Google Search grounding、文件操作、shell、web fetch；Headless 模式（CI/CD，JSON/stream-JSON）；GEMINI.md 项目记忆；会话 checkpoint | 已核实（SR-14 / SR-15） |
| 架构特点 | 终端优先；MCP 内建为核心扩展机制；可信文件夹 + 执行确认；多层沙箱（macOS Seatbelt / Linux gVisor·runsc·LXC·LXD·Docker·Podman / Windows Native Sandbox icacls） | 已核实（SR-15） |
| 部署形态 | 本地 CLI（npx/npm/brew/MacPorts/conda），云端 Gemini 模型 | 已核实（SR-14） |
| 集成方式 | 原生 MCP（GitHub/Slack/DB 等）、Google Search grounding、GitHub Action 集成 | 已核实（SR-14 / SR-15） |
| 定价模式 | 免费 OAuth 额度（60 req/min、1000 req/day）+ 付费 Gemini API | 已核实（SR-14） |
| 优势 | MCP-first 设计、1M 上下文、多层沙箱、Headless JSON 利于 CI、免费额度低门槛 | 综合归纳（SR-15） |
| 局限 | Google 云依赖（数据至 Google）；重度使用超免费额度后价格可能匹配/超 Claude；IDE 插件滞后 | 已核实（SR-15） |
| 对本项目的参考价值 | MCP-first、Headless JSON 流式、多层沙箱矩阵、GEMINI.md 记忆机制可借鉴；Google 云锁定与 Vynth 定位不符 | 推断 |

#### 2.2.6 B6 - Vynth（本项目，对照基准 / 自研代表）

| 维度 | 内容 | 置信度 |
| --- | --- | --- |
| 产品定位 | 「terminal 里的代码合成器」，AI-Native Coding Terminal，支持 Plan/Vibe 双模式，把自然语言合成成代码（D1） | 已核实（D1 / D48） |
| 目标用户 | 终端开发者、vibe coding 使用者、本地优先偏好者 | 已核实（D1 / D48） |
| 核心能力 | agent loop（maxSteps 默认 8，D24）+ 内置工具（read_file/write_file/run_shell，D23）+ OpenAI 兼容 SSE LLM 客户端（D22）+ demo EchoProvider 离线（D22）+ 插件无头接入（D42/D51） | 已核实（D22 / D23 / D24 / D51） |
| 架构特点 | 自研轻量 ANSI 渲染器（非 ink，放弃 yoga.wasm）+ 流式直写逃生舱（stream-escape-hatch，D29）；pnpm workspace + turbo（D1/D3）；@vynth/* 包组织（D49） | 已核实（D1 / D28 / D29 / D49） |
| 部署形态 | Bun + `bun build --compile` 单二进制，目标 20–40MB，当前约 61MB（含 react 残留，D52/D53/D54） | 已核实（D52 / D53 / D54） |
| 集成方式 | 仅环境变量配置（不读配置文件，D14）；插件 loader 已通过 `-p/--plugin` 接入无头模式（D42/D51）；McpClient 已就绪但未并入 CLI（D38/D49） | 已核实（D14 / D38 / D49 / D51） |
| 定价模式 | 自研，默认 DeepSeek（D14，与文档 X1/X2 冲突） | 已核实（D14）；默认值冲突见 §5.2 U-04 |
| 优势 | 单二进制本地优先、零依赖分发、Vibe/Plan 双模式、插件无头可扩展、demo 即开即体验 | 综合归纳（D48 / D54） |
| 局限 | run_shell 宿主权限、无进程/网络硬隔离（仅 VYNTH_NET 软开关，D34/D53/D54）；MCP 未接入 CLI；单二进制体积超标；仅环境变量无审计/合规配置层 | 已核实（D34 / D53 / D54） |
| 对本项目的参考价值 | 作为对照基准，其"单二进制 + 本地优先 + 自研 ANSI + 插件无头"已落地；主要待补强在沙箱硬隔离、MCP CLI 接入、体积、配置合规层 | 推断 |

### 2.3 关键技术能力横向事实

> 不评分、不排序，仅按能力维度横陈各方案事实。Vynth（B6）事实标注 D 编号出处；竞品标注来源章节。

| 能力维度 | B1 Claude Code | B2 OpenCode | B3 Aider | B4 Codex CLI | B5 Gemini CLI | B6 Vynth（本项目） |
| --- | --- | --- | --- | --- | --- | --- |
| 终端形态 / TUI | Ink（React + Yoga，虚拟列表）SR-20 | TUI（OpenTUI/SolidJS/Bubble Tea）SR-07 | 终端 + 可选浏览器 UI SR-09 | 全屏 TUI（含 /theme）SR-12 | 终端优先 SR-14 | 自研轻量 ANSI + 逃生舱 D28/D29 |
| Agent loop / 工具 | gather→act→verify 单线程；5 类工具 SR-01 | Provider 无关 loop；20+ 工具 SR-07 | chat loop + repo-map + edit applier SR-08 | ReAct loop 沙箱内；apply_patch SR-22 | agent loop + 内置工具 SR-15 | agent loop（maxSteps=8）+ 3 内置工具 D24/D23 |
| 上下文 / 项目记忆 | CLAUDE.md；92% 自动 compaction SR-02 | AGENTS.md / Skills / MCP SR-06 | repo-map（tree-sitter）SR-08 | AGENTS.md；上下文压缩 SR-22 | GEMINI.md；/compress SR-15 | 无项目记忆文件（仅环境变量）D14 |
| 部署 / 分发 | 本地 CLI + 云模型；native 100MB SR-04 | 本地优先单二进制/多端 SR-05 | 本地 CLI（pip）SR-09 | 本地 CLI（npm，Node 22+）SR-13 | 本地 CLI（npx/npm）SR-14 | Bun 单二进制，61MB（目标 20–40MB）D52/D53/D54 |
| 模型策略 | 仅 Anthropic（Claude）SR-03 | 75+ provider（含本地）SR-06 | 模型无关（多厂商+Ollama）SR-09 | 仅 OpenAI SR-13 | 仅 Gemini SR-14 | OpenAI 兼容 SSE，默认 DeepSeek D14/D22 |
| 沙箱 / 隔离 | bubblewrap(Linux)/seatbelt(macOS) 开源沙箱 SR-21 | 权限系统 + PTY；硬隔离公开少 SR-07 | 依赖 Git 审计，无 OS 级沙箱 SR-09 | Seatbelt(Linux Docker+iptables) SR-11/SR-13 | Seatbelt/gVisor/LXC/Docker/Podman/icacls SR-15 | 设计信任，无硬隔离；VYNTH_NET 软开关 D34/D53/D54 |
| 扩展（MCP / 插件） | MCP + hooks + subagents SR-01 | MCP Server + Skills SR-07 | 有限 MCP SR-15 | MCP（CLI+IDE 共享）SR-22 | 原生 MCP-first SR-14 | McpClient 就绪未并入 CLI；plugins 无头已接入 D38/D49/D51 |
| 商业模式 / 成本 | 订阅/API 用量 SR-03 | 免费 MIT，自备 Key SR-05 | 免费 Apache，付 token SR-09 | 免费 Apache，付 OpenAI SR-13 | 免费 OAuth + 付费 API SR-14 | 自研，无许可成本 D48 |

---

## 3. 对比：对比矩阵与加权评分

> **四段式「对比」段**。在 §2 事实基础上建立对比矩阵，赋予权重并打分。权重由主理人在调研目标中显式指定（已冻结），本报告直接采用。

### 3.1 对比矩阵

> 评分含义：本矩阵评估各标杆对 **Vynth（本地优先单二进制、TS、agent-loop+工具、自研 ANSI、Provider 无关）** 的「借鉴价值」。1 = 严重不符合，3 = 基本满足但有局限，5 = 完美契合。**每行权重之和 = 1.00**（主理人指定）。

| 评估维度 | 权重 | 权重理由（主理人指定） | B1 Claude Code | B2 OpenCode | B3 Aider | B4 Codex CLI | B5 Gemini CLI |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 场景契合度 | 0.30 | 与本项目核心场景（vibe coding TUI 终端 Agent）匹配的重要性 | 5 | 5 | 4 | 4 | 4 |
| 技术成熟度 | 0.20 | 标杆方案工程成熟度的参考权重 | 5 | 4 | 5 | 4 | 4 |
| 集成难度（反向） | 0.15 | 越高=越易融入 Vynth 的 TS 单二进制架构（可直接学习/借用） | 3 | 5 | 3 | 3 | 4 |
| 成本（反向） | 0.15 | 越高=采用该范式成本越低（避免云锁定/许可费） | 2 | 5 | 5 | 3 | 4 |
| 合规可控性 | 0.20 | 越高=越契合本地优先/自托管/无供应商锁定 | 2 | 5 | 5 | 3 | 3 |
| **加权总分** | **1.00** | — | **3.65** | **4.80** | **4.40** | **3.50** | **3.80** |

**评分标尺**：每项 1~5 分，1 = 严重不符合，3 = 基本满足但存在明显局限，5 = 完美契合。

**打分依据摘要（每项）**：
- 场景契合度：B1/B2 完美契合终端 Agent 范式（B2 还多模型中立+本地优先）；B3 偏 git-diff 编辑范式、B4/B5 偏云模型，契合度略低。
- 技术成熟度：B1/B3 最成熟（长期部署/基准验证）；B2/B4/B5 活跃但较新。
- 集成难度（反向）：B2 开源 TS 可直接学习/借用得 5；B1 闭源仅参考得 3；B3 Python 范式不同得 3；B4 开源但 Rust/OpenAI 锁定得 3；B5 开源 TS 得 4。
- 成本（反向）：B2/B3 免费+自备 Key 得 5；B1 云 SaaS 成本高得 2；B4 OpenAI 锁定得 3；B5 免费额度+付费得 4。
- 合规可控性：B2/B3 开源+本地+无锁定得 5；B1 闭源云模型得 2；B4/B5 开源但云模型锁定得 3。

### 3.2 评分结论

> 基于 §3.1 加权总分，形成分层结论。每层结论引用得分作为依据。

- **优先借鉴**：**OpenCode（B2，4.80）** 与 **Aider（B3，4.40）**。理由：二者均为开源、本地优先、模型中立/Provider 无关、无供应商锁定，在「成本（反向）」与「合规可控性」维度均得 5 分，与 Vynth「本地优先 + 单二进制 + 可自托管」定位高度一致；其中 OpenCode 的 TS + Plan/Build 双模式 + Provider 抽象层与 Vynth 技术栈最贴近，Aider 的 repo-map 上下文工程与 Git 审计轨迹是补充借鉴点。
- **部分借鉴**：**Claude Code（B1，3.65）**、**Gemini CLI（B5，3.80）**、**Codex CLI（B4，3.50）**。借鉴点：B1 的 agent-loop / Plan 模式 / 6 级权限 / 上下文压缩 / 流式 TUI（Ink 虚拟列表）范式，以及其开源沙箱（bubblewrap/seatbelt）思路；B5 的 MCP-first 设计、Headless JSON 流式、多层沙箱矩阵、GEMINI.md 记忆；B4 的审批模式分级、apply_patch、Seatbelt/Docker 沙箱、OpenTelemetry 审计日志。不借鉴的部分：三者的闭源云模型默认路径 / 厂商锁定（B1 Anthropic、B4 OpenAI、B5 Gemini），在合规可控性（B1=2、B4=3、B5=3）与成本（B1=2）维度显著偏低，不应作为 Vynth 默认交付形态。
- **不借鉴（否决）**：**以「闭源云后端 + 厂商锁定模型」作为 Vynth 的默认交付形态**（对应 B1/B4/B5 的 SaaS 默认路径）。否决理由：在合规可控性（B1=2、B4=3、B5=3）与成本（B1=2）维度显著偏低，与 Vynth「本地优先 + 单二进制 + 可自托管 + Provider 无关（默认 DeepSeek）」的项目定位直接冲突（D1/D14/D48）；Vynth 应保留自托管/多 provider 能力，仅将竞品的云模型作为可选接入而非默认锁定。

### 3.3 方案组合分析

| 组合方式 | 覆盖哪些能力 | 未覆盖能力 | 组合复杂度 | 总体成本估算 |
| --- | --- | --- | --- | --- |
| OpenCode（开源架构范式 + Plan/Build + Provider 抽象）+ Claude Code（沙箱/权限/流式 TUI 思路）+ Aider（repo-map/上下文压缩/Git 轨迹） | 本地优先范式、OS 级沙箱、流式渲染、上下文工程、Git 审计、多 provider | MCP-first 深度生态（需补 Gemini CLI 思路）、配置合规层 | 中 | 低（均为开源可借鉴，无许可费；工程人力为主） |

> 结论：单一方案无法覆盖 Vynth 全部需求——开源本地优先范式（OpenCode/Aider）解决可控性/成本，Claude Code 解决沙箱/权限/流式渲染范式，Gemini CLI 解决 MCP-first/Headless 流式。建议组合借鉴，而非单一采纳。

---

## 4. 建议：取舍决策支持

> **四段式「建议」段**。基于 §2 事实 + §3 对比，给出可被 `business-architect` 直接采用的建议。本节是建议而非最终裁决，最终边界由业务架构师冻结。

### 4.1 自研 / 采购 / 复用边界建议

| 能力项 | 建议方式 | 建议依据 | 候选方案 / 系统 | 关键前提 |
| --- | --- | --- | --- | --- |
| TUI 渲染（自研 ANSI + 逃生舱） | 自研/复用（已有底座） | Vynth 已自研轻量 ANSI 渲染器 + stream-escape-hatch（D28/D29），ADR-0003 明确不用 ink（D52）；对标 Claude Code 用 Ink 但放弃 yoga.wasm，逃生舱模式规避每 token 全树重渲染（SR-20） | Vynth 现有 tui 包 | 保留自研；可借鉴 Ink 的虚拟列表/增量更新思路优化逃生舱 |
| 单二进制分发（Bun） | 复用（已有底座） | Vynth 已用 `bun build --compile`（D2/D52）；Bun 单二进制行业常态 60–100MB（Claude Code Native 100MB，SR-04），但 Vynth 目标更激进 20–40MB | Bun compile | 需清理 react 残留以逼近目标体积（见 §5.1 R-02） |
| Agent loop + 内置工具 | 自研（已有底座） | Vynth 已自研 engine（D24，maxSteps=8；D23 三工具）；参考 OpenCode Plan/Build 与 Aider 的上下文工程 | Vynth engine 包 | 可引入 Plan/Vibe 外的上下文压缩与 repo-map 思路 |
| MCP 客户端 | 复用（已有底座）+ 待接入 | Vynth McpClient 已就绪（stdio JSON-RPC，D38）但未并入 CLI（D49）；对标 Claude Code/Gemini CLI MCP-first | Vynth mcp 包 → CLI agent 工具集 | 需 business-architect 冻结接入优先级与协议版本（§5.2 U-05） |
| 插件系统 | 复用（已有底座） | plugins loader 已通过 `-p/--plugin` 接入无头模式（D42/D51）；参考 OpenCode Skills/AGENTS.md | Vynth plugins 包 | TUI 内插件加载暂缓（见 §4.2） |
| Sandbox 隔离 | 自研（增强，高优先级） | Vynth 当前 run_shell 宿主权限、无硬隔离（D34/D53/D54）；对标 Claude Code bubblewrap/seatbelt（SR-21）、Codex Seatbelt/Docker（SR-11）、Gemini CLI 多层沙箱（SR-15） | OS 级沙箱（bubblewrap/seatbelt） | 需评估跨平台（macOS/Linux/Windows）支持成本（§5.3 D-01） |
| Provider / LLM 抽象 | 复用（已有）+ 扩展 | Vynth 已 OpenAI 兼容 SSE（D22），默认 DeepSeek（D14）；参考 OpenCode 75+ provider 抽象（SR-06） | Vynth engine llm.ts | 统一默认值文档/代码冲突（§5.2 U-04） |
| 项目记忆文件（CLAUDE.md/AGENTS.md/GEMINI.md 类） | 建议新增 | 三家竞品均有项目记忆文件机制（SR-02/SR-06/SR-15）；Vynth 当前仅环境变量、无项目记忆（D14） | 新增配置文件/约定 | 与「仅环境变量」设计权衡（§5.3 D-03） |

### 4.2 MVP 范围建议

> 对齐 Vynth 当前 P0/P1：单二进制 TUI、agent loop+内置工具、demo provider、插件无头接入；暂缓 MCP CLI 接入、TUI 内插件、配置文件、联网硬隔离（D54）。

| 功能（对齐用户诉求 / Vynth 现状） | 建议 MVP？ | 理由 |
| --- | --- | --- |
| 单二进制 TUI | ✅ | 技术可行，标杆（B1–B5）均支持单二进制/本地 CLI；Vynth 已实现（D28/D29/D52） |
| agent loop + 内置工具（read/write/shell） | ✅ | Vynth engine 已落地（D23/D24）；竞品均以此为核心 |
| demo provider（无 key 离线体验） | ✅ | EchoProvider 已就绪（D22）；降低首次体验门槛，标杆无直接等价但价值明确 |
| 插件无头接入（`-p/--plugin`） | ✅ | 代码已实现并经 api/overview 确认（D51）；无头模式可交付 |
| 暂缓：MCP CLI 接入 | ⚠️ 暂缓（路线图） | McpClient 已就绪但未并入 CLI（D38/D49）；建议后续接入以补生态劣势（§5.1 R-04） |
| 暂缓：TUI 内插件加载 | ⚠️ 暂缓 | 当前插件仅无头接入（D54）；TUI 内加载涉及信任模型联动（§5.3 D-04） |
| 暂缓：配置文件（仅环境变量） | ⚠️ 暂缓 | 当前仅环境变量（D14），符合 ADR-0003 设计；企业合规需补配置层（§5.1 R-03） |
| 暂缓：联网硬隔离 | ⚠️ 暂缓 | 当前 VYNTH_NET 软开关（D34/D53）；需 OS 级沙箱升级（§5.1 R-01） |

### 4.3 技术栈参考建议

| 技术层 | 推荐方案 | 替代方案 | 选择理由 |
| --- | --- | --- | --- |
| 运行时 / 单二进制分发 | Bun `bun build --compile`（Vynth 已用，D52） | Node SEA（Rolldown 打包，114MB，SR-19）/ Deno compile（565MB 无 tree-shaking，SR-18） | Bun 单二进制 60–100MB 远小于 Deno（9x 更小，SR-18），冷启动 50–100ms 优于运行时（SR-18）；Vynth 已采用 |
| TUI 渲染 | 自研轻量 ANSI + stream-escape-hatch（Vynth 已用，D28/D29） | Ink（React + Yoga，Claude Code 用，SR-20） | 自研规避 ink 的 React 重渲染开销与 yoga.wasm 依赖（ADR-0003，D52）；逃生舱直写应对高频 token 流式（对标 Ink 增量更新思路） |
| LLM 客户端 | OpenAI 兼容 SSE（Vynth 已用，D22） | Vercel AI SDK 多 provider 抽象（OpenCode 用，SR-06） | 已落地；可参考 OpenCode 的 Provider 抽象扩展多模型，保持 Provider 无关 |
| 沙箱隔离 | bubblewrap（Linux）/ seatbelt（macOS）（参考 Claude Code，SR-21） | Docker（Codex 用，SR-11，较重）/ gVisor（Gemini CLI，SR-15） | OS 级原语轻量、无容器开销；对标 Claude Code 开源沙箱思路 |
| MCP 传输 | stdio JSON-RPC（Vynth 已用 2024-11-05，D38） | Streamable HTTP（MCP 2025-03-26 起，SR-16） | stdio 适合本地子进程（Vynth 本地优先）；建议后续升级到 MCP 2025-11-25 稳定版（§5.2 U-05） |

---

## 5. 风险与待确认项

> **四段式「风险」段**。列出调研中发现的主要风险、不确定信息、待业务架构师进一步裁决的依赖项，以及仍需人工补充调研的部分。

### 5.1 主要风险清单

| 编号 | 风险描述 | 触发条件 | 影响范围 | 严重程度 | 缓解建议 |
| --- | --- | --- | --- | --- | --- |
| R-01 | Sandbox 无硬隔离导致越权/数据泄露：Vynth run_shell 以宿主权限运行、无进程/网络硬隔离，仅 VYNTH_NET 软开关（D34/D53/D54） | 用户加载恶意插件（动态 import 执行任意代码，D53）或 prompt injection 触发 run_shell 读取 SSH key / 写入敏感路径 | 宿主文件系统与凭据泄露、横向移动 | 高 | 引入 OS 级沙箱（bubblewrap/seatbelt，参考 Claude Code SR-21、Codex SR-11、Gemini CLI SR-15）；将 VYNTH_NET 软开关升级为硬网关；插件执行沙箱化；参考 AI coding agent 威胁模型（"致命三要素"：私密数据访问+不可信内容+对外通信，SR-23） |
| R-02 | 单二进制体积超标：目标 20–40MB，当前约 61MB（含 react 残留，D52/D53/D54） | bundle 含 react/ink 残留或大型依赖未清理 | 分发/下载体验下降、冷启动变慢 | 中 | 清理 react 残留（ADR-0003 已明确去 ink/react，D52）；tree-shaking + minify；评估 `--bytecode` 预编译（注意：触发实测 bytecode 对长运行服务有损、对 CLI 冷启动有益，SR-18）；外置大依赖 |
| R-03 | 仅环境变量配置的审计/合规缺口：Vynth 不读配置文件（D14），无配置审计轨迹 | 企业合规要求配置可追溯/审计 trail | 无法满足企业审计与配置管理要求 | 中 | 保留环境变量为主（符合设计），增加可选配置文件/审计日志层，参考 Codex 的 OpenTelemetry 日志（SR-11）与 Claude Code 合规日志平台（SR-21） |
| R-04 | MCP 未接入导致的生态劣势：McpClient 已就绪但未并入 CLI（D38/D49） | 用户期望接入 GitHub/Slack/DB 等 MCP 服务 | 扩展能力弱于 Claude Code/Gemini CLI（MCP-first），生态竞争力下降 | 中 | 将 McpClient 并入 CLI agent 工具集（路线图），对齐 MCP 2025-11-25 稳定版（§5.2 U-05），参考 Gemini CLI MCP-first 设计（SR-14） |
| R-05 | 默认 LLM 端点文档与代码不一致（X1/X2 冲突）导致用户混淆 | 用户按 README 配 OpenAI 实际走 DeepSeek（D14/D50/D51/D53 冲突） | 配置误解、支持成本、信任下降 | 低–中 | 统一文档与代码默认值，由 business-architect 裁决以代码（DeepSeek）还是文档（OpenAI）为准（§5.2 U-04） |

### 5.2 待确认项（需主理人 / 业务方反馈）

| 编号 | 待确认项 | 不确定性说明 | 若无法确认的备选路径 |
| --- | --- | --- | --- |
| U-01 | OpenCode 官方仓库地址与归档/迁移状态 | 多源冲突：opencode.ai 官网与 cnblogs 指向 `github.com/anomalyco/opencode`（SR-05/SR-06）；另有来源称 `opencode-ai/opencode` 已归档、以 "Crush" 在 Charm 继续，或 `kodrunhq/opencode`（SR-参考社区争议） | 以 opencode.ai 官网为准，标注社区仓库迁移争议；不影响本报告结论（仅影响精确引用 URL） |
| U-02 | Vynth 单二进制能否压到 20–40MB 目标 | 当前 61MB 含 react 残留（D52/D53/D54），清理后真实体积需工程验证 | 若无法达标，按 60–100MB 行业常态（SR-04/SR-18）重新设目标，降级为"体积优化"而非硬指标 |
| U-03 | Vynth 是否计划引入 OS 级沙箱（bubblewrap/seatbelt） | 取决于安全优先级与 macOS/Linux/Windows 跨平台支持成本（§5.3 D-01） | 若暂不引入，至少将 VYNTH_NET 软开关升级为硬网关 + 插件沙箱化作为过渡 |
| U-04 | 默认 LLM 端点应以代码（DeepSeek）还是文档（OpenAI）为准 | X1/X2 冲突：代码 D14 默认 DeepSeek，文档 D1/D50/D51/D54 写 OpenAI（material_digest §3） | 由 business-architect 冻结；建议以代码为准并统一文档，避免用户混淆（R-05） |
| U-05 | MCP 接入优先级与协议版本（2024-11-05 vs 2025-11-25） | Vynth McpClient 用 2024-11-05（D38）；MCP 最新稳定版 2025-11-25（SR-17） | 若暂缓接入，保留 2024-11-05 兼容；若接入，评估升级到 2025-11-25 |

### 5.3 需业务架构持续关注的依赖项

| 编号 | 依赖项 | 说明 | 建议关注阶段 |
| --- | --- | --- | --- |
| D-01 | 若采用 OS 级沙箱增强，需评估跨平台（macOS/Linux/Windows）支持成本 | 沙箱引入影响单二进制分发与平台兼容性（R-01） | 高层架构设计 §5.2 / 安全设计 |
| D-02 | 信任边界 / 安全模型需嵌入安全设计 | 见 R-01，Vynth 当前设计信任（D53/D54）需升级为硬隔离 | 安全设计阶段 |
| D-03 | 配置体系（仅环境变量 vs 加配置文件）决策 | 影响企业合规与审计（R-03），与 ADR-0003「仅环境变量」设计权衡 | business-architect 冻结 |
| D-04 | 插件生态（无头 vs TUI 内）的信任模型 | 需与 sandbox 设计联动（D-01/D-02），插件动态 import 执行任意代码（D53） | business-architect / 安全设计 |

---

## 6. 关键来源目录

> 集中列出全部调研所使用的公开资料、官方文档、社区仓库、分析报告等。每条来源不低于 URL 粒度，关键来源给出具体章节或段落。

**硬指标**：≥ 3 条来源，覆盖每家标杆（B1–B5）；关键数据指定来源段落。

| 编号 | 来源类型 | 标题 / 名称 | URL / 路径 | 相关章节 | 最后访问日期 |
| --- | --- | --- | --- | --- | --- |
| SR-00 | 内部摘要 | Vynth 资料摘要 v0.1（material_digest） | `.workbuddy/output/material_digest.md` | 全文（D 编号出处） | 2026-07-24 |
| SR-01 | 官方博客 | Building agents with the Claude Agent SDK（Anthropic） | https://claude.com/blog/building-agents-with-the-claude-agent-sdk | B1 §核心能力/架构特点 | 2026-07-24 |
| SR-02 | 技术拆解 | Understanding How Claude Code Works（virtuslab） | https://virtuslab.com/blog/ai/how-claude-code-works/ | B1 §架构特点/上下文 | 2026-07-24 |
| SR-03 | 指南 | Claude Code: The complete guide（datanorth） | https://datanorth.ai/blog/claude-code-ai-coding-assistant-guide-2025 | B1 §部署/定价/SWE-bench | 2026-07-24 |
| SR-04 | 社区文章 | Claude Code Native Build: 100MB Binary（dev.to） | https://dev.to/frr149/claude-code-native-build-100mb-binary-to-ditch-node-for-good-2gi6 | B1 §局限（单二进制体积）；§4.3 | 2026-07-24 |
| SR-05 | 官方站点 | OpenCode 官网 | https://www.opencode.ai/ | B2 §全部（stars/license/隐私） | 2026-07-24 |
| SR-06 | 架构分析 | OpenCode: 153K Stars 开源编码 Agent（ai-pulse） | https://ai-pulse-pi.vercel.app/post/analysis-2026-05-03-opencode | B2 §架构/Provider/双模式 | 2026-07-24 |
| SR-07 | 架构分析 | OpenCode: Open Source Coding Agent（pyshine） | http://pyshine.com/Opencode-Open-Source-Coding-Agent/ | B2 §架构/TUI/MCP/LSP | 2026-07-24 |
| SR-08 | 技术文档 | Aider（marovi.ai） | https://marovi.ai/Aider | B3 §架构/repo-map/edit | 2026-07-24 |
| SR-09 | 技术分析 | Aider: The Open-Source AI Pair Programmer（botmonster） | https://botmonster.com/ai/aider-model-agnostic-ai-pair-programmer/ | B3 §repo-map/Architect Mode/SWE-bench | 2026-07-24 |
| SR-10 | 官方文档 | OpenAI Codex CLI – Getting Started | https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started | B4 §审批模式/安装 | 2026-07-24 |
| SR-11 | 官方文档 | Running Codex safely at OpenAI | https://openai.com/index/running-codex-safely/ | B4 §沙箱/网络策略/遥测 | 2026-07-24 |
| SR-12 | 官方文档 | Codex CLI Features（OpenAI Developers） | https://developers.openai.com/codex/cli/features | B4 §TUI/子代理/MCP | 2026-07-24 |
| SR-13 | 开源仓库 | OpenAI Codex CLI（GitHub） | https://github.com/MadcowD/codex | B4 §安全模型/平台沙箱 | 2026-07-24 |
| SR-14 | 开源仓库 | Gemini CLI（google-gemini GitHub） | https://github.com/google-gemini/gemini-cli | B5 §全部（特性/安装/MCP） | 2026-07-24 |
| SR-15 | 技术分析 | Gemini CLI（similarlabs） | https://similarlabs.com/zh/p/gemini-cli-ai-developer-tool | B5 §沙箱/MCP/Headless | 2026-07-24 |
| SR-16 | 规范 | Model Context Protocol – Transports（stdio） | https://modelcontextprotocol.io/specification/2025-06-18/basic/transports | §2.2/§4.3（MCP stdio） | 2026-07-24 |
| SR-17 | 术语库 | Model Context Protocol（ai-solutions.wiki） | https://ai-solutions.wiki/glossary/model-context-protocol | §2.2/§5.2 U-05（MCP 版本） | 2026-07-24 |
| SR-18 | 技术基准 | Reducing Single Binary Size by 9x: Deno→Bun（zenn） | https://zenn.dev/dyoshikawa/articles/deno-to-bun-single-binary | §3.1/§4.3/§5.1 R-02（单二进制体积） | 2026-07-24 |
| SR-19 | 基准仓库 | bun-vs-node-sea-startup（yyx990803 GitHub） | https://github.com/yyx990803/bun-vs-node-sea-startup | §3.1/§4.3（冷启动/体积基准） | 2026-07-24 |
| SR-20 | 技术拆解 | Inside Claude Code – CLI, Commands & Terminal UI（Ink） | https://y-agent.github.io/inside-claude-code/08-cli-commands-ui.html | B1 §TUI/§2.3/§4.3（Ink 管线） | 2026-07-24 |
| SR-21 | 官方工程 | Making Claude Code more secure and autonomous（sandboxing） | https://www.anthropic.com/engineering/claude-code-sandboxing | B1 §沙箱/§2.3/§5.1 R-01 | 2026-07-24 |
| SR-22 | 模式目录 | Codex CLI — Framework（agentpatternscatalog） | https://www.agentpatternscatalog.org/compositions/codex-cli | B4 §架构/审批/MCP（注：其"Rust"说法与官方 npm/Node 文档冲突，见 §5.2 U-01 说明） | 2026-07-24 |
| SR-23 | 安全分析 | AI Coding Agent Security: Threat Models（knostic.ai） | https://www.knostic.ai/blog/ai-coding-agent-security | §5.1 R-01（威胁模型） | 2026-07-24 |

---

## 7. 硬指标清单

> 汇总本模板所有章节的硬指标，供自动校验与人工审核使用。

| 章节 | 硬指标项 | 当前状态 | 备注 |
| --- | --- | --- | --- |
| §1 | 调研问题已收敛为 ≥ 3 条可执行问题 | ✅ | Q1–Q6 共 6 条，对齐 6 大调研方向 |
| §2.1 | 标杆系统 ≥ 3 家，含 ≥ 1 家头部 SaaS | ✅ | B1/B4/B5 头部 SaaS（Claude Code/Codex/Gemini CLI） |
| §2.1 | 标杆系统 ≥ 1 家开源或自研代表 | ✅ | B2/B3 开源（OpenCode/Aider）+ B6 Vynth 自研 |
| §2.2 | 每家标杆有独立详述卡片 | ✅ | B1–B5 完整 10 维度卡片 + B6 Vynth 对照卡 |
| §2.3 | 关键能力横向事实无遗漏 | ✅ | 8 能力维度横陈 B1–B6，Vynth 标 D 编号出处 |
| §3.1 | 对比矩阵含 5 维度 + 权重 + 评分 | ✅ | 权重和 = 1.00（主理人指定），B1–B5 逐项打分 |
| §3.2 | 评分结论含优先/部分/不借鉴三层 | ✅ | 优先(OpenCode/Aider)/部分(Claude Code/Gemini CLI/Codex)/不借鉴(云锁定默认形态) |
| §4.1 | 自研/采购/复用边界有明确建议 | ✅ | 8 项能力边界表，含候选方案与关键前提 |
| §4.2 | MVP 范围建议与用户诉求对齐 | ✅ | 对齐 Vynth P0/P1，含暂缓项与理由 |
| §5.1 | 主要风险 ≥ 3 条，有缓解建议 | ✅ | R-01~R-05 共 5 条，均含触发/影响/严重度/缓解 |
| §6 | 关键来源可追溯（URL / 章节） | ✅ | SR-00~SR-23 共 24 条，覆盖 B1–B5，关键数据指定段落 |
| 全文 | 明确区分事实 / 推断 / 建议 / 风险 | ✅ | §2 事实（含置信度）/§3 对比/§4 建议/§5 风险 四段分明 |
| 全文 | 不存在编造来源或占位符 | ✅ | 无尖括号占位符 / 待验证标记 / 示例前缀；待确认项以 U-xx 明确标注 |

---

## 附录 A：中间确认自检报告（按协议 §2.4 在 4 个关键章节产出后执行）

> 依据 `skills/aicoding-team-bootstrap/protocols/intermediate_confirmation.md` §2.1 + §2.3，在 §1.2 / §2.1 / §3.1 / §5.2 四个关键点完成自检。结论：**4 次均未命中触发标准**，故未发起 `[中间确认]` 阻塞；以下为反向验证 3 问的证据记录，供主理人 G2/G3 审核追溯。

### A.1 自检点 1：§1.2 调研问题收敛

- **§2.1 判定**：未命中。收敛方向由用户诉求（D0：vibe coding TUI，类比 Claude Code/OpenCode/Codex）与 team-lead 调研目标（6 方向）显式给定，无 ≥2 种合理分歧理解需用户裁决。
- **§2.3 反向验证 3 问**：
  - Q1（返工成本）：调研问题列表仅为本报告 §1 文档表格，不涉及代码/架构产出；若推翻仅重写 §1（约 1 页），切换成本 ≈ 0 人月。证据：§1 纯文档表格，无下游产物依赖。
  - Q2（用户感知）：不能。调研问题列表是 G2 内部研究文档，不进入产品/合同/合规。证据：属内部调研报告。
  - Q3（与诉求一致）：一致。直接引用 team-lead 注入：「请围绕以下方向展开调研...1. 竞品对标...2. 终端 TUI 渲染技术...3. 单二进制分发...4. Agent 循环与工具系统...5. MCP 与插件扩展...6. 安全/信任模型」。§1 的 Q1–Q6 直接映射此 6 方向。
- **结论**：未命中，不发起。

### A.2 自检点 2：§2.1 标杆候选名单

- **§2.1 判定**：未命中。标杆名单由用户诉求显式点名（Claude Code/OpenCode/Codex）+ team-lead 补充（Aider/Gemini CLI）给定；≥3 家含头部 SaaS（B1/B4/B5）+ 开源代表（B2/B3/B6），均由上游指定，无方案分歧。
- **§2.3 反向验证 3 问**：
  - Q1（返工成本）：标杆名单若推翻，仅重写 §2.1/§2.2/§2.3 表格（约 4 页），不影响下游架构代码；切换成本 ≈ 0 人月。证据：标杆盘点纯文档。
  - Q2（用户感知）：不能。标杆名单为内部调研，不进产品/合同/合规。证据：G2 内部文档。
  - Q3（与诉求一致）：一致。直接引用用户诉求「类似 Claude Code、OpenCode、Codex 等终端编程工具」+ team-lead「（以及可补充的 Aider、Cursor CLI、Gemini CLI 等）」；B1–B5 直接覆盖。
- **结论**：未命中，不发起。

### A.3 自检点 3：§3.1 权重分配

- **§2.1 判定**：未命中。权重（场景契合度 0.30 / 技术成熟度 0.20 / 集成难度反向 0.15 / 成本反向 0.15 / 合规可控性 0.20，和=1.00）由上游 team-lead 在调研目标中显式指定，属「已冻结」指定，非本研究员单方裁决。
- **§2.3 反向验证 3 问**：
  - Q1（返工成本）：权重若推翻，仅重算 §3.1 矩阵权重列与加权总分（约 1 表），切换成本 ≈ 0 人月。证据：仅矩阵数值。
  - Q2（用户感知）：不能。权重是评估方法论内部参数，不进产品。证据：G2 文档。
  - Q3（与诉求一致）：权重由上游显式指定，与用户诉求无冲突（用户诉求未指定权重，上游代为指定）。直接引用 team-lead：「对比矩阵 5 维度（场景契合度 0.30 / 技术成熟度 0.20 / 集成难度反向 0.15 / 成本反向 0.15 / 合规可控性 0.20），权重和=1.00」。
- **结论**：未命中，采用指定权重，不发起。

### A.4 自检点 4：§5.2 待确认项整理（最终复核）

- **§2.1 判定**：未命中。待确认项（U-01~U-05）是对「外部信息不可得 / 项目内部冲突待裁决」的标记，非方案分歧决策；按协议归入 §5.2 待确认项处理，不触发中间确认。
- **§2.3 反向验证 3 问**：
  - Q1（返工成本）：待确认项本身为「未决」状态，不影响已定稿内容；若后续补充确认，仅更新 §5.2 表格 + 可能微调 §4 建议。返工范围 ≈ 1 表。证据：§5.2 独立表格。
  - Q2（用户感知）：待确认项不进产品；但其中 U-04（默认 LLM 端点代码/文档冲突）会在用户配置时感知到文档与代码不一致——此属「待确认项」而非「已做决策」，按协议列 §5.2 转主理人，不触发中间确认。证据：U-04 源于项目内部 X1/X2 冲突。
  - Q3（与诉求一致）：待确认项是对「外部信息不可得 / 内部冲突」的标记，非与用户诉求冲突；U-04 源于 material_digest X1/X2，由 business-architect 裁决。
- **结论**：未命中，待确认项按 §5.2 列出转主理人，不发起。

> 备注：本研究员在全程未做「已冻结」之外的单方裁决；所有评分均为评估而非授权，最终业务边界由 business-architect 冻结。
