# AICoding 架构设计 · UserStory

> 本文档为《AICoding 架构设计》核心产物之一，定位为**产品需求与用户故事（UserStory）**。
> 上游输入：《高层架构设计》v0.1（G3 已审核通过，采用方案 A：MVP 维持设计信任模型、OS 级沙箱推迟至完整版）；
> 下游输出：驱动《系统设计》《部署设计》《安全设计》的具体功能实现。
>
> 本文档是 G4 阶段的《UserStory》唯一 Owner 产物，与 system-architect 的《系统设计》共同构成 G4 审核对象。所有角色、场景、功能边界、MVP 范围均严格继承《高层架构设计》已冻结结论，不新增超出上游范围的功能需求。

---

## 1. 业务背景与价值

### 1.1 业务背景

- **当前业务现状（行业 / 产品 / 用户规模）**：Vynth 是本地优先、单二进制的 vibe coding TUI 终端编程工具（类比 Claude Code / OpenCode / Codex）。v0.1.0 已实现 agent loop、内置工具（read / write / shell）、demo、插件无头接入与自研 ANSI TUI；行业对标覆盖 Claude Code、OpenCode、Aider、Codex CLI、Gemini CLI 五家标杆（research_report §2.1–§2.3）。用户规模为终端开发者、自动化/CI 使用者、插件开发者与企业安全合规 reviewer 五类角色（高层架构 §2.1）。
- **触发本次需求的事件（新场景 / 痛点修复）**：补齐安全与生态短板、冻结能力边界、明确 MVP / 完整版演进。当前短板为：sandbox 无进程/网络硬隔离（P1）、MCP 未并入 CLI（P2）、单二进制体积 61MB 超标（P3）、仅环境变量配置致合规缺口（P4）、默认 LLM 端点文档与代码不一致（P5）（高层架构 §1.1 / §2.2）。
- **本系统在产品矩阵中的位置**：在“本地优先终端编程工具”矩阵中承担核心 Agent 引擎职责，与上游 LLM Provider、下游 sandbox / 插件形成完整业务闭环；不做云端后端、不做厂商锁定模型默认、不做多租户 SaaS（高层架构 §4.2）。

### 1.2 行业方案

> 同类功能、痛点的行业标杆系统及解决方案（对齐 research_report §2 / §3）。

| 标杆 | 厂商 / 社区 | 部署形态 | 与 Vynth 相关的关键方案 | 借鉴结论 |
| --- | --- | --- | --- | --- |
| B1 Claude Code | Anthropic（闭源 SaaS + 本地 CLI） | 本地 CLI + 云端模型 | agentic loop、6 级权限、Plan 模式、Ink TUI、bubblewrap/seatbelt OS 级沙箱 | 部分借鉴（沙箱/权限/流式范式）；不借鉴闭源云模型默认 |
| B2 OpenCode | Anomaly 社区（开源 MIT） | 本地优先单二进制 | TypeScript + Provider 无关、Plan/Build 双模式、MCP/Skills | 优先借鉴（加权 4.80，最契合本地优先+开源） |
| B3 Aider | Paul Gauthier（开源 Apache 2.0） | 本地 CLI + Git | repo-map 上下文工程、Git 审计轨迹 | 优先借鉴（加权 4.40） |
| B4 Codex CLI | OpenAI（开源 Apache 2.0） | 本地 CLI + 云端模型 | 三档审批、Seatbelt/Docker 沙箱、OpenTelemetry 审计 | 部分借鉴（审批分级/沙箱思路） |
| B5 Gemini CLI | Google（开源 Apache 2.0） | 本地 CLI + 云端模型 | MCP-first、多层沙箱、Headless JSON | 部分借鉴（MCP-first/Headless） |
| B6 Vynth（自研） | vynth 团队 | 本地优先单二进制 | 自研 ANSI + 逃生舱、OpenAI 兼容 SSE、Plan/Vibe、插件无头 | 基准（本期冻结能力边界） |

**行业共识与 Vynth 差异**：竞品普遍提供 OS 级沙箱（Claude Code bubblewrap/seatbelt、Codex Seatbelt/Docker、Gemini CLI 多层沙箱），Vynth MVP（方案 A）维持设计信任模型（软 `VYNTH_NET`），OS 级硬隔离推迟至完整版——该决策经 G3 阶段 [中间确认] 已冻结为方案 A（高层架构 §C.2）。

### 1.3 方案收益与价值

| 功能模块 | 预期价值收益 | 量化标准（来源） |
| --- | --- | --- |
| 单二进制本地优先分发（F1） | 零依赖即装即用、无云成本、无供应商锁定 | 单二进制体积 MVP ≤ 61MB、完整版目标 ≤ 40MB（N2 / D52） |
| 默认 DeepSeek + OpenAI 兼容 SSE（F6/F7） | 模型中立、成本显著低于闭源 SaaS | 单任务 token 成本较 GPT-4o 类降 ≥ 80%（高层架构 §1.3） |
| 冷启动低延迟（N1） | 启动到可用会话体验顺滑 | 冷启动 P95 ≤ 150ms（N1 / D52） |
| 无 Key demo 离线体验（F8） | 首次启动即可体验，降低上手门槛 | 无 Key 进入 demo 率 100%（EchoProvider，D22） |
| 设计信任模型 + 完整版 OS 沙箱兜底（F10/F15） | 合规可控、安全短板可兜底 | MVP 高危操作策略覆盖 = 软隔离；完整版 100% 策略留痕（V1/V4） |
| 插件无头接入（F9） | 可扩展、生态可生长 | 已接入 1 类无头插件加载路径（D51） |

### 1.4 术语清单

> 统一文档中专有名词的中英文对照与含义，与 system-architect 术语表对齐。

| 术语 | 含义 |
| --- | --- |
| Vynth | 本地优先、单二进制的 vibe coding TUI 终端编程工具（本项目） |
| TUI | Terminal User Interface，终端用户界面；本项目为自研轻量 ANSI 渲染，非 ink |
| vibe coding | 以自然语言驱动、AI 合成代码的终端编程范式 |
| agent loop | 智能体循环：组装消息 → LLM 流式补全 → 工具调用 → 回填，maxSteps 默认 8（D24） |
| StreamEvent | 跨层唯一协议：`{type:'token'}` / `{type:'tool'}` / `{type:'done'}`（D18） |
| ToolResult | 工具统一返回：`{ok, output, error?}`（D18） |
| sandbox / safeResolve | 工具执行唯一出口；`safeResolve` 做 cwd 内解析 + 符号链接二次校验（D34） |
| VYNTH_NET | 网络软开关，默认开启；`0/off/false/no` 关闭，仅软隔离非硬边界（D14/D34） |
| 设计信任模型 | MVP 信任边界 = 宿主权限，无 OS 级进程/网络硬隔离（方案 A，D53/D54） |
| Plan / Vibe 双模式 | `VYNTH_MODE`：`plan`（规划）/`vibe`（默认，直接合成）（D14/D18） |
| EchoProvider | 无 API Key 时的离线回显 Provider，支持 demo 触发工具调用（D22） |
| McpClient | MCP stdio JSON-RPC 客户端，已就绪但未并入 CLI（完整版 F12，D38） |
| 插件无头接入 | `-p/--plugin` 经 `loadPlugin` + `activate` 注入工具，仅无头模式（F9，D51） |
| 单二进制 | `bun build --compile` 产出单一可执行文件，Bun 运行时内嵌（D52） |
| Catppuccin 主题 | mocha / latte 双主题调色板（theme.ts，D30） |

---

## 2. 范围与边界

### 2.1 系统内模块及功能

> 一级功能清单（MVP 必做项 + 完整版系统内待建项）。MVP（P0）含 F1–F11；F12–F15 属系统内但本期不实现（完整版）。

| 一级模块 | 二级模块 | 功能项（MVP 必做） | 说明 |
| --- | --- | --- | --- |
| CLI 入口 | 启动 / 帮助 | F1 单二进制启动、F11 退出码/帮助/版本 | Bun compile 单二进制；`-v/--version`、`-h/--help`、`-g/--goal`、`-m/--mode`、`-p/--plugin`（D10） |
| TUI 渲染 | ANSI 渲染 / 主题 | F2 ANSI 渲染+逃生舱、F3 双模式+主题 | 自研轻量 ANSI（非 ink），StreamArea 行内直写；mocha/latte（D28/D29/D30） |
| Agent 引擎 | agent loop | F4 agent loop（maxSteps=8） | 流式 token/tool 事件循环（D24） |
| 内置工具 | 文件/Shell | F5 read_file / write_file / run_shell | 经 sandbox 执行（D23） |
| LLM 客户端 | SSE / 默认 / demo | F6 OpenAI 兼容 SSE、F7 默认 DeepSeek 对齐、F8 EchoProvider | 流式 chat/tool_calls；默认 `https://api.deepseek.com/v1` + `deepseek-chat`（D22） |
| 插件系统 | 无头接入 | F9 无头插件接入 | `-p/--plugin` 加载并激活（D42/D51） |
| 沙箱守卫 | 越界守卫 | F10 safeResolve + symlink 二次校验 | 软 `VYNTH_NET`；无 OS 级硬隔离（D34） |
| 配置中心 | 环境变量 | N3 7 项环境变量体系 | 仅 `process.env`，不读配置文件（D14） |
| MCP 客户端（完整版） | CLI 接入 | F12 McpClient 并入 agent 工具集 | 本期不实现（O1） |
| 插件系统（完整版） | TUI 内插件 | F13 TUI 内插件加载 | 本期不实现（O2） |
| 配置中心（完整版） | 配置合规层 | F14 可选配置文件 + 审计 | 本期不实现（O3） |
| 沙箱守卫（完整版） | OS 级硬隔离 | F15 bubblewrap/seatbelt + 硬网关 | 本期不实现（O4，方案 A 推迟） |

### 2.2 系统外模块及功能

> 当前系统**不覆盖**的功能，及其原因（对齐高层架构 §6.1.2–§6.1.4）。

| 编号 | 不做的事 | 原因 | 后续计划 |
| --- | --- | --- | --- |
| O1 | MCP CLI 接入（F12） | McpClient 已就绪但未并入 CLI，生态补强非 MVP 阻塞（D38/D49/R-04） | 完整版 |
| O2 | TUI 内插件加载（F13） | 信任模型联动未定，当前仅无头模式（D54/D-04） | 完整版 |
| O3 | 配置文件 / 配置合规审计层（F14） | ADR-0003 明确仅环境变量；企业合规需补（R-03） | 完整版 |
| O4 | OS 级沙箱（进程 + 网络硬隔离）（F15） | 跨平台成本（macOS/Linux/Windows），不阻塞 MVP（R-01/U-03）；已触发并冻结为方案 A | 完整版 |
| O5 | 多租户 SaaS / 云端后端 | 与本地优先单二进制定位冲突，用户诉求未要求（D0/D48） | 不做（定位决议） |
| O6 | 预编译发行包 / 插件市场 / 自动更新 | 分发简化优先，v0.1.0 范围边界明确排除（D54） | 后续版本另议 |

### 2.3 外部依赖

| 依赖系统 | 提供方 | 依赖能力 | 接入方式 | 接口人 |
| --- | --- | --- | --- | --- |
| LLM Provider（OpenAI 兼容 SSE） | DeepSeek / OpenAI / 自建 | chat / tool_calls 流式补全 | HTTPS SSE（fetch POST /chat/completions） | 外部 Provider；`assertSafeEndpoint` 拒绝向非 localhost 明文 http 发 Key（D22） |
| MCP Server（完整版） | 社区 / 自建 | tools/list、tools/call | stdio JSON-RPC（protocolVersion 2024-11-05） | 外部；完整版接入（D38） |
| 宿主文件系统 | OS | read / write / run_shell | 进程内调用 sandbox（safeResolve） | 本机 OS |
| 宿主 Shell | OS | run_shell 命令执行 | `spawn sh -c`（win32 用 `cmd /c`），默认 30s 超时 SIGKILL | 本机 OS |
| `@vynth/core` | 自研 | config / events / logger / errors | 进程内 import | 内部团队 |
| `@vynth/tui` | 自研 | ANSI 渲染 + 逃生舱 | 进程内 import | 内部团队 |
| `@vynth/plugins` | 自研 | `-p` 动态 import 加载 | 进程内 `import()` | 内部团队 |

---

## 3. 功能清单

> **定位**：全景骨架表，进入“角色 / 场景 / US”之前先看到完整功能版图。本表与《高层架构设计》§6.3 功能清单互查一致（F1–F15 编号、优先级、MVP/完整版归属均不变），不新增超出上游范围的功能需求。

### 3.1 功能清单结构

| 一级模块 | 二级模块 | 功能项 | 优先级（P0/P1/P2） | MVP 范围 | 完整版范围 | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| CLI 入口 | 单二进制启动 | F1 单二进制启动（Bun compile，`--version`/`--help`） | P0 | ✅ | ✅ | D2/D10 |
| TUI 渲染 | ANSI 渲染 + 逃生舱 | F2 自研 ANSI，高频 token 直写，非 ink | P0 | ✅ | ✅ | D28/D29 |
| TUI 渲染 | 双模式 + 主题 | F3 Plan / Vibe + mocha / latte | P0 | ✅ | ✅ | D14/D30 |
| Agent 引擎 | agent loop | F4 maxSteps=8，token/tool 事件循环 | P0 | ✅ | ✅ | D24 |
| 内置工具 | read / write / shell | F5 经 sandbox 执行 | P0 | ✅ | ✅ | D23 |
| LLM 客户端 | OpenAI 兼容 SSE | F6 流式 chat / tool_calls 聚合 | P0 | ✅ | ✅ | D22 |
| LLM 客户端 | 默认 DeepSeek 对齐 | F7 代码为准，文档统一 | P0 | ✅ | ✅ | 对齐 V5 / D4 |
| LLM 客户端 | demo EchoProvider | F8 无 Key 离线 | P0 | ✅ | ✅ | D22 |
| 插件系统 | 无头插件接入 | F9 `-p/--plugin` 加载并激活 | P0 | ✅ | ✅ | D42/D51 |
| 沙箱守卫 | 越界守卫 | F10 safeResolve + symlink 二次校验 | P0 | ✅ | ✅ | 对齐 V1（部分）/ D34 |
| CLI 入口 | 退出码 / 帮助 | F11 0 / 2 / 非 0 语义 | P0 | ✅ | ✅ | D10/D51 |
| MCP 客户端 | CLI 接入 | F12 McpClient 并入 agent 工具集 | P1 | ❌ | ✅ | 对齐 V2 / O1 |
| 插件系统 | TUI 内插件 | F13 TUI 加载（信任模型联动） | P2 | ❌ | ✅ | O2 |
| 配置中心 | 配置合规层 | F14 可选配置文件 + 审计 | P2 | ❌ | ✅ | 对齐 V4 / O3 |
| 沙箱守卫 | OS 级硬隔离 | F15 bubblewrap / seatbelt + 硬网关 | P1 | ❌（方案 A 推迟） | ✅ | 对齐 V1 / O4 |
| 非功能 | 冷启动时延 | N1 冷启动 P95 ≤ 150ms | P0 | ✅ | ✅ | D52 |
| 非功能 | 单二进制体积 | N2 MVP 维持 ≤ 61MB 并启动优化，完整版目标 ≤ 40MB | P0 | ✅ | ✅ | D52/D53 |
| 非功能 | 配置体系 | N3 仅环境变量配置（7 变量） | P0 | ✅ | ✅ | D14（ADR-0003） |

**范围说明**：所有 P0 级功能（F1–F11 + N1/N2/N3）均在 MVP 范围内标记为 ✅，即 **P0 ＝ MVP**；F12–F15 延后至完整版（含 OS 级沙箱 F15，方案 A 已冻结推迟）。本表与高层架构 §6.3 完全一致，互查通过。

---

## 4. 角色与场景

### 4.1 角色清单

| 角色 | 业务身份 | 主要操作 | 核心关注点 |
| --- | --- | --- | --- |
| R1 甲方决策者（产品 / 技术负责人） | Vynth 团队负责人 | 范围裁决 / 路线图 / 资源投入 | ROI 与合规可控：本地优先无锁定、分发成本可控、安全短板是否兜底 |
| R2 最终用户 A（终端开发者 - TUI 交互） | 本地开发者 | 启动 TUI、输入 goal、审阅流式补全与工具执行 | 流式渲染不卡顿、核心路径 ≤ 3 步启动会话 |
| R3 最终用户 B（自动化 / CI 使用者 - 无头模式） | DevOps / 脚本 | `-g` 无头执行、管道集成、插件加载 | 无头模式稳定、退出码语义明确、可脚本化 |
| R4 企业安全 / 合规 reviewer | 安全 / 合规 reviewer | 审计配置、评估数据驻留与泄露面 | 危险操作（读 SSH key / 联网）有策略与留痕 |
| R5 插件开发者 | 第三方 / 内部插件作者 | 编写 / 加载插件（`-p`） | 插件契约清晰、信任边界明确、宿主权限可控 |

> 五类角色均独立可识别，覆盖决策方、终端用户、受影响方；与《高层架构设计》§2.1 核心角色关注点对齐。

### 4.2 关键场景清单

| 编号 | 角色 | 触发条件 | 期望结果 | 频率（日均 / QPS） |
| --- | --- | --- | --- | --- |
| S1 | R2 | 开发者在 TTY 执行 `vynth` 且无 `-g` | 进入主交互屏，输入 goal 后流式补全 + 工具回显 | 高（人均多会话/日） |
| S2 | R2 | 未设置 `VYNTH_API_KEY` 直接启动 | 自动进入 EchoProvider demo，离线可体验 | 中（首装/试用） |
| S3 | R3 | CI 脚本执行 `vynth -g "目标"` | 标准输出流式 token/tool，退出码 0/2/非 0 可被脚本判断 | 高（CI 流水线，QPS≈1/进程） |
| S4 | R5 | 执行 `vynth -g "目标" -p 插件路径` | 插件加载并注册工具，管道化输出扩展能力 | 低-中（插件开发期） |
| S5 | R2 | 设置 `VYNTH_MODE=plan` 或 `VYNTH_THEME=latte` | 切换 Plan 模式或 latte 主题，调色板重绘 | 低（按需） |
| S6 | R2 / R4 | `run_shell` 或 `read_file` 指向 cwd 外 / 符号链接逃逸 | 越界守卫拒绝并回显 SandboxError | 低（异常路径） |
| S7 | R2 | 配置 `VYNTH_LLM_BASE_URL` / `VYNTH_MODEL` 等 7 变量 | 以 DeepSeek 默认值统一接入真实 LLM | 中（接入期一次） |
| S8 | R1 / R4 | MVP 交付验收 | 拿到 MVP 交付物 + 已知局限清单（软隔离/无 MCP/体积/仅环境变量） | 低（里程碑） |

### 4.3 角色 → 旅程 → 功能 映射总览

> 下图由 `diagrams-generator` 能力（Graphviz + PingFang SC 字体）生成，展示五类角色驱动八条用户旅程、并映射到 F1–F11 MVP 功能的关系。

![Vynth MVP 用户旅程映射](pic/userstory/us_journey.png)

*图：角色（R1–R5）→ 用户旅程（US-1–US-8）→ MVP 功能（F1–F11）映射。实线表示驱动 / 覆盖关系。来源：Graphviz 生成，源文件 `pic/userstory/us_journey.dot`。*

---

## 5. 用户旅程（UserStory）

> 每条 UserStory 均按 5.1.1 ~ 5.1.7 的 7 个小节展开（业务场景 / 业务流程 / UE 原型 / 业务逻辑 / 数据描述 / 验收标准 / 外部集成接口）。全部 8 条 US 覆盖 MVP 功能 F1–F11，不新增超出《高层架构设计》范围的功能。

### 5.1 US-1：终端开发者首次启动 TUI 进入 vibe coding 会话

#### 5.1.1 业务场景

- **视角**：最终用户 A（终端开发者，R2）。
- **描述逻辑**：用户在配备 TTY 的终端中执行 `vynth`（未带 `-g`）启动 TUI；在 ≤ 3 步内完成「启动 → 输入 goal → 审阅流式结果」的核心路径。设置 `VYNTH_API_KEY` 后接入真实 LLM（默认 DeepSeek），未设置则回退 EchoProvider demo（见 US-2）。会话中 agent loop 驱动内置工具经 sandbox 执行，结果以 ANSI 流式直写呈现。

#### 5.1.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 用户在 TTY 环境且已设置 `VYNTH_API_KEY`，When 执行 `vynth` 启动 TUI，Then 系统在冷启动 P95 ≤ 150ms 内进入主交互屏并以默认 vibe 模式就绪。
  - Given 主交互屏已就绪，When 用户输入 goal 并回车提交，Then agent loop 调用 LLM 流式补全，token 经 StreamArea 行内直写、工具调用回显至历史区。
  - Given 会话连续多轮，When 达到 maxSteps=8 或 LLM 返回 done，Then 会话完成并等待下一轮输入；用户按 Ctrl-C 干净退出并恢复终端 raw mode。

#### 5.1.3 UE 原型

```
┌──────────────────────────────────────────────┐
│ Vynth · mocha            [vibe]   Ctrl-C 退出  │
├──────────────────────────────────────────────┤
│ > 给 src/agent.ts 增加重试逻辑                  │  ← 用户输入 goal
│ ──────────────────────────────────────────── │
│ (token) 好的，我将为 agent-loop 增加指数退避…   │  ← StreamArea 行内直写
│ (tool)  read_file  src/agent.ts               │  ← 工具回显
│ (tool)  write_file src/agent.ts               │
│ (token) 已完成，已添加 retryWithBackoff。       │
└──────────────────────────────────────────────┘
```

#### 5.1.4 业务逻辑

- **视角**：业务系统（时序）。
  1. `main` 解析参数 → 非 `-g` 且 `isTTY` → `loadConfig({mode})`。
  2. `startTui(config)`：`palette(theme)` → `createProvider(config)` → `builtinTools(cwd,{networkAllowed})` → 进入 readline + raw mode。
  3. 用户提交 → `runAgent(provider, tools, goal)`：组装 `[system, user(goal)]`，循环 `provider.chat` 收集 token/pendingTool。
  4. 有 `pendingTool` → `yield tool` → `tools.run` → 回填 `[assistant, tool]`，直至 done 或 maxSteps。
  5. `tool.run` 经 `sandbox`（safeResolve + runCommand）执行；结果 `ToolResult` 回显。
  6. 结束 → `cleanup` 恢复 raw mode、close readline。

#### 5.1.5 数据描述

- 输入：`goal`（用户自然语言）、`VynthConfig`（mode/llmBaseUrl/apiKey/model/theme/sandbox/dataDir）。
- 流转：`ChatMessage[]`（system+user+assistant+tool）→ `StreamEvent`（token/tool/done）→ `ToolResult` → 终端 stdout（ANSI 转义）。
- 关键不变量：`StreamEvent` 为跨层唯一协议；`ToolResult` 统一 `{ok,output,error?}`；sandbox 是工具执行唯一出口（高层架构 §5.2 / D49）。

#### 5.1.6 验收标准 AC

- **正常路径**：Given 用户在 TTY 执行 `vynth` 且已配置 `VYNTH_API_KEY`，When 主屏出现后输入 goal 并回车，Then 系统在 150ms 内开始流式输出 token、工具调用回显可见、会话可连续多轮。
- **正常路径（无 Key 回退）**：Given 用户未配置 `VYNTH_API_KEY`，When 执行 `vynth`，Then 系统以 EchoProvider 进入 demo 且主屏可交互（详见 US-2）。
- **异常路径（非 TTY）**：Given 用户在非 TTY 环境执行 `vynth`（无 `-g`），When 程序检测非 TTY，Then 打印提示并以退出码 2 退出，引导改用 `-g` 无头模式。
- **异常路径（LLM 错误）**：Given LLM Provider 返回错误或网络中断，When `OpenAiProvider` 解析 SSE 失败，Then 系统回显 LlmError 且不以非零方式静默崩溃，可重试或退出。
- **异常路径（超时）**：Given agent loop 达到 maxSteps=8，When 仍未 done，Then 结束本轮并提示用户，不无限循环。

#### 5.1.7 外部集成接口

- LLM Provider（OpenAI 兼容 SSE，默认 DeepSeek）：`POST {baseUrl}/chat/completions`，逐行解析 `data:` SSE 帧（D22）。
- 宿主文件系统 / Shell：经 `@vynth/sandbox` 进程内调用（D34）。

---

### 5.2 US-2：无 API Key 用户零配置进入 demo 离线体验

#### 5.2.1 业务场景

- **视角**：最终用户 A（R2），首次试用者。
- **描述逻辑**：用户未设置 `VYNTH_API_KEY`，直接启动 `vynth`（或 `-g`），系统在 0 配置下自动切换 EchoProvider，进入离线 demo 体验，验证工具调用链路，无需任何外部 LLM 账号。

#### 5.2.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 用户未设置 `VYNTH_API_KEY`，When 启动 `vynth`，Then `createProvider` 返回 EchoProvider，进入 demo 模式且主屏提示「demo 模式」。
  - Given 处于 demo 模式，When 用户输入含 `demo-tool` 的 goal 且存在工具，Then EchoProvider 调用首个工具并填示例参数，回显工具结果与中文回显文本。
  - Given 处于 demo 模式，When 用户输入普通 goal，Then EchoProvider 回显中文「（demo）收到目标：…」并不发起任何外部网络请求。

#### 5.2.3 UE 原型

```
┌──────────────────────────────────────────────┐
│ Vynth · mocha        [demo · 离线]  Ctrl-C 退出 │
├──────────────────────────────────────────────┤
│ > 用 demo-tool 打个招呼                          │
│ (token) （demo）收到目标：用 demo-tool 打个招呼   │
│ (tool)  read_file（示例参数）回显                │
│ (token) 这是离线 demo，未连接任何 LLM。          │
└──────────────────────────────────────────────┘
```

#### 5.2.4 业务逻辑

- **视角**：业务系统。`createProvider(config)`：`apiKey` 为空 → 返回 `EchoProvider`（D22）。`EchoProvider.chat`：goal 含 `demo-tool` 且存在工具 → 调用首个工具填示例参数；否则回显中文文本。无 SSE、无网络。

#### 5.2.5 数据描述

- 输入：`goal` 字符串；`apiKey=''`（D14 默认值）。
- 流转：`EchoProvider` 产出 `StreamEvent`（token / 可选 tool）→ `ToolResult` → stdout。无外部数据传输。

#### 5.2.6 验收标准 AC

- **正常路径**：Given 环境无任何 `VYNTH_API_KEY`，When 用户启动 `vynth`，Then 系统在 150ms 内进入 demo 模式且主屏可见「demo」标识。
- **正常路径（工具触发）**：Given demo 模式且工具集非空，When goal 含 `demo-tool`，Then 回显一次工具调用结果与回显文本。
- **异常路径（无工具）**：Given demo 模式且未加载插件（无自定义工具），When goal 含 `demo-tool`，Then 仅回显中文文本，不报错中断。
- **异常路径（误配 Key 但无效）**：Given 用户设置了无效 `VYNTH_API_KEY`，When 启动，Then 系统仍尝试 `OpenAiProvider` 并在 SSE 失败时回显 LlmError（不静默进入 demo）。

#### 5.2.7 外部集成接口

- 无外部依赖（demo 离线）。真实 LLM 接入见 US-7 / US-1。

---

### 5.3 US-3：自动化 / CI 使用者以无头模式执行目标并消费退出码

#### 5.3.1 业务场景

- **视角**：最终用户 B（R3，DevOps/脚本）。
- **描述逻辑**：CI 脚本或管道以 `vynth -g "目标"` 执行无头模式，标准输出流式 token/tool，结束后以退出码语义（0 正常 / 2 用法错误 / 非 0 运行期错误）返回，便于脚本判断成功与否。

#### 5.3.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given CI 脚本执行 `vynth -g "重构 util.ts"`，When agent loop 正常完成，Then 标准输出流式 token/tool，进程以退出码 0 结束。
  - Given 用户传入非法参数（如未知 flag），When `parseArgs` 解析失败，Then 打印用法并以退出码 2 退出。
  - Given 运行期工具执行失败或 LLM 不可达，When `runAgent` 抛出运行期错误，Then 进程以非零退出码（非 0/非 2）结束并回显错误。

#### 5.3.3 UE 原型

```
$ vynth -g "给 util.ts 加类型注解"
(tool) read_file util.ts
(tool) write_file util.ts
(token) 已为 util.ts 补全类型注解。
$ echo $?
0
```

#### 5.3.4 业务逻辑

- **视角**：业务系统。`main`：`parseArgs` → 有 `goal` → `runHeadless`：`loadConfig()` → `createProvider` → `builtinTools(cwd,{networkAllowed})` → `runAgent` 循环 `token`/`tool` 事件直写 stdout。非 TTY 时若无 `-g` 走 `process.exit(2)` 提示（D10）。

#### 5.3.5 数据描述

- 输入：`goal` 字符串经 `-g`；输出：stdout 流式 `StreamEvent`；进程退出码。
- 退出码契约：`0` 正常 / `2` 用法错误 / 非 0 运行期错误（D51）。

#### 5.3.6 验收标准 AC

- **正常路径**：Given CI 执行 `vynth -g "目标"`，When 会话正常完成，Then 退出码为 0 且 stdout 含流式 token/tool 文本。
- **正常路径（管道）**：Given 将 stdout 管道至文件，When 会话完成，Then 文件完整包含 token 与工具调用记录，无 ANSI 转义污染（无头模式不依赖 TTY 渲染）。
- **异常路径（用法错误）**：Given 传入未知参数，When `parseArgs` 失败，Then 退出码为 2 并打印用法。
- **异常路径（运行期错误）**：Given 工具执行抛 SandboxError 且未恢复，When `runAgent` 终止，Then 退出码为非 0（非 2）且回显错误文本。
- **异常路径（非 TTY 无 goal）**：Given 非 TTY 执行 `vynth` 且无 `-g`，When 检测非 TTY，Then 退出码 2 并提示改用 `-g`。

#### 5.3.7 外部集成接口

- LLM Provider（OpenAI 兼容 SSE）；宿主 Shell/文件系统（sandbox）。

---

### 5.4 US-4：插件开发者以无头模式加载自定义插件扩展工具集

#### 5.4.1 业务场景

- **视角**：插件开发者（R5）。
- **描述逻辑**：插件开发者编写导出 `pluginName` 与 `activate(reg)` 的模块，以 `vynth -g "目标" -p 插件路径` 在无头模式加载；`loadPlugin` 动态 import 插件并 `activate` 注册工具，扩展 agent 可用工具集，管道化输出。

#### 5.4.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 开发者执行 `vynth -g "用 hello 工具向世界问好" -p packages/plugins/examples/hello-plugin.ts`，When `loadPlugin(abs)` 成功，Then `activate(reg)` 注册 `hello` 工具并进入 agent loop。
  - Given 插件被正常激活，When goal 命中 `hello` 工具，Then agent 调用该工具并将结果回显至 stdout。
  - Given 插件路径无效或导出缺失，When `loadPlugin` 校验失败，Then 抛出 `PluginError` 并以非零退出码结束。

#### 5.4.3 UE 原型

```
$ vynth -g "用 hello 工具向世界问好" -p plugins/examples/hello-plugin.ts
(tool) hello  name=世界
(token) 你好，世界！
$ echo $?
0
```

#### 5.4.4 业务逻辑

- **视角**：业务系统。`runHeadless`：若 `pluginPath` → `loadPlugin(abs)`（`import(entryPath)`，要求导出 `pluginName`+`activate`，否则 `PluginError`）→ `plugin.activate(tools)` 注册工具 → `runAgent`（D10/D42）。信任边界：插件经动态 `import()` 执行任意代码，宿主完整权限（D53）。

#### 5.4.5 数据描述

- 输入：插件文件路径（绝对化）；输出：注册后的 `ToolDef` 进入 `ToolRegistry`；agent 经 `tools.run` 调用。
- 信任边界数据：插件可读取 `VYNTH_API_KEY` 等环境变量（D53），本期无沙箱化（设计信任模型）。

#### 5.4.6 验收标准 AC

- **正常路径**：Given 合法插件路径且导出符合契约，When `-p` 加载，Then 工具被注册并可在 goal 中调用，退出码 0。
- **正常路径（多工具）**：Given 插件 `activate` 注册多个工具，When goal 依次命中，Then 各工具被正确调用并回显。
- **异常路径（导出缺失）**：Given 插件未导出 `pluginName` 或 `activate`，When `loadPlugin` 校验，Then 抛 `PluginError` 且退出码非 0。
- **异常路径（路径无效）**：Given `-p` 指向不存在的路径，When `import()` 失败，Then 回显错误并以非零退出码结束。
- **异常路径（激活抛错）**：Given 插件 `activate` 内部抛错，When 激活阶段，Then 捕获并回显 `PluginError`，不进入 agent loop。

#### 5.4.7 外部集成接口

- `@vynth/plugins` 动态 import；宿主文件系统（读取插件文件）。插件市场/签名（O6）本期不做。

---

### 5.5 US-5：用户切换 Plan / Vibe 双模式与 Catppuccin 主题

#### 5.5.1 业务场景

- **视角**：最终用户 A（R2）。
- **描述逻辑**：用户通过环境变量 `VYNTH_MODE`（plan/vibe，默认 vibe）与 `VYNTH_THEME`（mocha/latte，默认 mocha）切换运行模式与配色；切换后调色板重绘，核心交互不变。

#### 5.5.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 用户设置 `VYNTH_MODE=plan`，When 启动 `vynth`，Then 系统以 plan 模式进入（规划优先，不直接落盘）。
  - Given 用户设置 `VYNTH_THEME=latte`，When `palette(theme)` 应用，Then 全屏以 latte 浅色调色板重绘（`draw()`）。
  - Given 用户未设置二者，When 启动，Then 采用默认值 vibe + mocha。

#### 5.5.3 UE 原型

```
$ VYNTH_MODE=plan VYNTH_THEME=latte vynth
┌──────────────────────────────────────────────┐
│ Vynth · latte           [plan]   Ctrl-C 退出   │  ← 浅色主题 + plan 模式
└──────────────────────────────────────────────┘
```

#### 5.5.4 业务逻辑

- **视角**：业务系统。`loadConfig`：mode 非 plan/vibe 则默认 vibe；theme `=== 'latte'` 则 latte 否则 mocha（D14）。`startTui`：`palette(theme)` 取 Catppuccin 调色板，`draw()` 用调色板重绘（D28/D30）。

#### 5.5.5 数据描述

- 输入：`VYNTH_MODE`、`VYNTH_THEME`（环境变量）。
- 流转：`VynthConfig.mode/theme` → `palette(theme)` → ANSI 真彩转义（`fg(hex)`/`bg(hex)`）。

#### 5.5.6 验收标准 AC

- **正常路径**：Given 设置 `VYNTH_MODE=plan`，When 启动，Then 模式标识显示 plan 且行为为规划优先。
- **正常路径（主题）**：Given 设置 `VYNTH_THEME=latte`，When 启动，Then 全屏以 latte 调色板渲染。
- **异常路径（非法模式）**：Given 设置 `VYNTH_MODE=bogus`，When 启动，Then 回落为默认 vibe 不报错。
- **异常路径（非法主题）**：Given 设置 `VYNTH_THEME=unknown`，When 启动，Then 回落为默认 mocha。

#### 5.5.7 外部集成接口

- 无外部依赖（纯本地配置 + ANSI 渲染）。

---

### 5.6 US-6：沙箱越界守卫拦截危险文件访问

#### 5.6.1 业务场景

- **视角**：最终用户 A（R2）/ 企业安全合规 reviewer（R4）。
- **描述逻辑**：内置工具的 `read_file` / `write_file` / `run_shell` 均经 `sandbox.safeResolve` 执行。当用户或 agent 尝试访问 cwd 之外路径、或经符号链接逃逸至沙箱外时，越界守卫拒绝并回显 `SandboxError`；`run_shell` 受 `VYNTH_NET` 软开关约束（默认开启，关闭则拒绝联网）。**MVP 为设计信任模型（软 `VYNTH_NET`），无 OS 级进程/网络硬隔离**——该已知局限在 US-8 明确。

#### 5.6.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 工具请求读取 `../secret.key`（cwd 外），When `safeResolve(cwd, target)` 解析后落在 cwd 外，Then 抛 `SandboxError` 并回显「路径越界」，工具返回 `{ok:false}`。
  - Given 工具经符号链接指向 cwd 外的真实文件，When `safeResolve` 做 `realpathSync` 二次校验发现逃逸，Then 抛 `SandboxError` 拒绝访问。
  - Given `VYNTH_NET=off` 且 `run_shell` 尝试联网，When `runCommand` 检测 `networkAllowed` 为 falsy，Then 直接返回 `{ok:false, error:'network blocked by sandbox policy'}`。

#### 5.6.3 UE 原型

```
(tool) read_file ../secret.key
→ SandboxError: 路径越界（解析结果不在 cwd 内）
(tool) write_file /etc/passwd
→ SandboxError: 路径越界
$ VYNTH_NET=off vynth -g "curl 外网"
(tool) run_shell curl https://example.com
→ {ok:false, error:'network blocked by sandbox policy'}
```

#### 5.6.4 业务逻辑

- **视角**：业务系统。`safeResolve`：先 `resolve(cwd, target)` 必须落在 cwd 内否则 `SandboxError`；再 `realpathSync` 解析符号链接二次校验，cwd 内 symlink 指向沙箱外 → `SandboxError`（D34，X4 已修复）。`runCommand`：`networkAllowed` falsy → 直接 `{ok:false}`；否则 `spawn sh -c`，默认 30s 超时 SIGKILL。

#### 5.6.5 数据描述

- 输入：工具参数中的 `target` 路径、`VYNTH_NET`、`cwd`。
- 流转：`safeResolve` → `fs` 读写或 `runCommand` → `ToolResult{ok,output,error?}`。
- 信任边界：仅软隔离；`run_shell` 在 `VYNTH_NET` 开启时仍可能联网（非硬网关）。

#### 5.6.6 验收标准 AC

- **正常路径（cwd 内）**：Given 工具请求读取 cwd 内合法文件，When `safeResolve` 通过，Then 正常返回文件内容 `{ok:true}`。
- **异常路径（cwd 外）**：Given 请求读取 cwd 外路径，When `safeResolve` 解析越界，Then 返回 `SandboxError` 且工具 `{ok:false}`。
- **异常路径（symlink 逃逸）**：Given 经 symlink 指向 cwd 外，When `realpathSync` 二次校验，Then 拒绝并回显 SandboxError（不读取目标）。
- **异常路径（联网被禁）**：Given `VYNTH_NET=off` 且工具尝试联网，When `runCommand` 检测，Then 返回 `network blocked by sandbox policy`。
- **已知局限（非硬隔离）**：Given `VYNTH_NET` 未关闭（默认开启），When `run_shell` 尝试联网，Then **允许联网**（软开关，非 OS 级硬隔离）——该行为在 US-8 / 安全设计明确记录，不为 MVP 默认硬隔离。

#### 5.6.7 外部集成接口

- 宿主文件系统 / Shell（进程内 sandbox）；OS 级硬隔离（F15）为完整版，本期不实现。

---

### 5.7 US-7：终端开发者配置默认 DeepSeek 端点与 7 项环境变量体系

#### 5.7.1 业务场景

- **视角**：最终用户 A（R2）。
- **描述逻辑**：用户通过 7 项环境变量（`VYNTH_API_KEY` / `VYNTH_MODEL` / `VYNTH_LLM_BASE_URL` / `VYNTH_MODE` / `VYNTH_THEME` / `VYNTH_NET` / `VYNTH_DATA_DIR`）接入并调优 LLM。代码默认 `VYNTH_LLM_BASE_URL=https://api.deepseek.com/v1`、`VYNTH_MODEL=deepseek-chat`（D14，已冻结为方案对齐 V5），文档与代码一致。

#### 5.7.2 业务流程

- **视角**：用户。
- **描述方式**（Given / When / Then）：
  - Given 用户设置 `VYNTH_API_KEY=sk-...`，When 启动，Then `createProvider` 返回 `OpenAiProvider` 并以默认 `https://api.deepseek.com/v1` + `deepseek-chat` 发起 SSE。
  - Given 用户未设置任何变量，When 启动，Then 采用全部代码默认值（DeepSeek 端点/模型、vibe 模式、mocha 主题、网络开启、cwd=process.cwd()、dataDir=~/.vynth）。
  - Given 用户传入 `VYNTH_LLM_BASE_URL=http://localhost:8787`（本地点 mock），When `assertSafeEndpoint` 校验，Then 允许（localhost 明文 http 例外）并连接本地服务。

#### 5.7.3 UE 原型

```
$ export VYNTH_API_KEY=sk-xxxx
$ vynth -g "解释 AgentOpts"
(tool) ...（真实 DeepSeek 流式补全）
$ vynth --help
  环境变量: VYNTH_API_KEY / VYNTH_MODEL / VYNTH_LLM_BASE_URL /
            VYNTH_MODE / VYNTH_THEME / VYNTH_NET / VYNTH_DATA_DIR
```

#### 5.7.4 业务逻辑

- **视角**：业务系统。`loadConfig(overrides?)`：仅读 `process.env`，不读配置文件（D14/ADR-0003）。`createProvider`：`apiKey` 非空 → `OpenAiProvider`（构造时 `assertSafeEndpoint`，拒绝向非 localhost 明文 http 发 Key，向非 api.openai.com 端点发 key 时告警）（D22）。

#### 5.7.5 数据描述

- 输入：7 项环境变量；输出：`VynthConfig` 驱动 provider/tools/tui。
- 数据流：env → `loadConfig` → `VynthConfig` → `createProvider` / `builtinTools` / `startTui`。

#### 5.7.6 验收标准 AC

- **正常路径（默认 DeepSeek）**：Given 仅设 `VYNTH_API_KEY`，When 启动并发送 goal，Then 请求发往 `https://api.deepseek.com/v1/chat/completions` 且 model=deepseek-chat。
- **正常路径（全部默认）**：Given 不设任何变量且有 Key，When 启动，Then 采用 DeepSeek 默认值，行为与文档一致（V5 达成）。
- **异常路径（明文 http 非 localhost）**：Given `VYNTH_LLM_BASE_URL=http://evil.com/v1` 且 Key 非空，When `assertSafeEndpoint`，Then 拒绝发送 Key 并告警/报错。
- **异常路径（未知端点告警）**：Given `VYNTH_LLM_BASE_URL=https://other.com/v1`，When 发 Key，Then 打印非 OpenAI 端点告警但仍允许（用户显式配置）。
- **异常路径（缺失 Key 但非 localhost）**：Given 未设 Key 且自定义非 localhost 端点，When 启动，Then 回落 EchoProvider demo（apiKey 为空判定优先）。

#### 5.7.7 外部集成接口

- LLM Provider（OpenAI 兼容 SSE）；`assertSafeEndpoint` 安全校验（D22）。

---

### 5.8 US-8：甲方决策者与安全合规 reviewer 验收 MVP 交付物与已知局限

#### 5.8.1 业务场景

- **视角**：甲方决策者（R1）/ 企业安全合规 reviewer（R4）。
- **描述逻辑**：MVP 交付后，决策方与合规方验收交付物（F1–F11 单二进制 TUI + agent loop + 内置工具 + demo + 插件无头 + 双模式主题 + 默认 DeepSeek 对齐 + 越界守卫），并收到**已知局限清单**：设计信任模型（软 `VYNTH_NET`，无 OS 级硬隔离）、MCP 未接入 CLI、单二进制体积 61MB、仅环境变量无配置审计层。这些局限对应完整版 F12–F15，供路线图决策。

#### 5.8.2 业务流程

- **视角**：验收方。
- **描述方式**（Given / When / Then）：
  - Given 决策方拿到 MVP 单二进制，When 对照功能清单（§3）逐项验证，Then F1–F11 全部可演示、退出码语义正确、demo 可离线运行。
  - Given 合规方评估安全模型，When 审阅信任边界文档（`VYNTH_NET` 软开关、插件宿主完整权限、无 OS 级隔离），Then 在验收报告中标注「R-01 高危操作无硬隔离」为已知局限、完整版以 F15 兜底。
  - Given 合规方要求审计能力，When 检查配置与操作留痕，Then 明确本期（方案 A）不提供操作审计留痕（F14 完整版），仅环境变量配置。

#### 5.8.3 UE 原型

```
MVP 验收清单（方案 A）
┌─────────────────────────────────────────────┐
│ [✓] F1 单二进制启动   [✓] F7 DeepSeek 对齐     │
│ [✓] F2 ANSI 渲染      [✓] F8 EchoProvider      │
│ [✓] F3 双模式+主题    [✓] F9 无头插件          │
│ [✓] F4 agent loop     [✓] F10 越界守卫         │
│ [✓] F5 内置工具       [✓] F11 退出码/帮助       │
│ [!] 已知局限：软隔离/无 MCP/体积 61MB/仅环境变量 │
└─────────────────────────────────────────────┘
```

#### 5.8.4 业务逻辑

- **视角**：业务系统（交付与文档）。MVP 交付物 = 单二进制 + 本文档功能清单 + 《系统设计》《安全设计》。已知局限来自高层架构 §6.1 Out-of-Scope 与 §2.2 痛点（P1 软隔离、P2 MCP/体积、P4 仅环境变量）。路线图：完整版 F12 MCP、F13 TUI 插件、F14 配置审计、F15 OS 沙箱（方案 A 冻结）。

#### 5.8.5 数据描述

- 输入：MVP 构建产物 + 三份设计文档（系统/部署/安全）。
- 流转：功能清单（§3）→ 验收勾选 → 已知局限清单 → 完整版路线图（F12–F15）。

#### 5.8.6 验收标准 AC

- **正常路径（功能达标）**：Given 决策方按 §3 逐项验收，When F1–F11 均可演示，Then 验收结论为 MVP 功能达标、可进入下一里程碑。
- **正常路径（局限透明）**：Given 合规方审阅信任边界，When 拿到已知局限清单，Then 清单明确标注 R-01 软隔离、R-03 仅环境变量、R-04 MCP 未接入、R-02 体积，且均映射到完整版 F12–F15。
- **异常路径（局限被误读为已修复）**：Given 验收文档未标注软隔离局限，When 合规方审阅，Then 文档必须显式标注「MVP 无 OS 级硬隔离，F15 完整版兜底」，不得写为已硬隔离。
- **异常路径（安全承诺越界）**：Given 任何 MVP 文档声称「默认高危操作被 OS 级隔离」，When 自检，Then 必须纠正为「设计信任模型（软 `VYNTH_NET`），OS 级硬隔离推迟至完整版 F15（方案 A）」。

#### 5.8.7 外部集成接口

- 无新增外部集成；依赖 LLM Provider、宿主 OS（同 US-1/US-3）。

---

## 6. 非功能性需求

### 6.1 易用性需求

> 操作便利性、UI 一致性、引导提示、错误反馈、无障碍支持等。

- **操作便利性**：核心路径「启动 → 输入 goal → 审阅流式结果」操作步数 ≤ 3 步（高层架构 §6.4）；非 TTY 环境引导改用 `-g` 无头模式；Ctrl-C / Ctrl-D 干净退出并恢复终端 raw mode（D28）。
- **UI 一致性**：Catppuccin 双主题（mocha/latte）统一调色板（`palette`/`fg`/`bg`/`reset`，D30）；TUI 与无头模式共用同一 agent loop，渲染层分离（高层架构 §5.2）；高频 token 走 StreamArea 行内直写，避免全树重渲染（D29）。
- **引导提示**：无 `VYNTH_API_KEY` 时自动进入 demo 并主屏提示「demo 模式」；`--help` 打印用法、退出码语义与 7 项环境变量（D10/D51）；工具/LLM 错误以 `ToolResult`/`VynthError` 结构化回显，不静默崩溃。
- **错误反馈**：`VynthError` 体系（ConfigError/LlmError/ToolError/SandboxError/PluginError）携带 code，错误经 stdout/stderr 可见（D15）；`run_shell` 30s 超时返回明确错误（D34）。
- **无障碍支持**：采用 ANSI 真彩转义呈现；本期未做屏幕阅读器适配与高对比模式（已知局限，完整版另议），不阻碍 MVP 核心可用性。

### 6.2 性能响应需求

> 关键接口响应时延（P50 / P90 / P99）、吞吐量（QPS / TPS）、并发用户数、数据规模上限等。

- **冷启动时延（基线已给定）**：冷启动 P95 ≤ 150ms（N1 / D52）。MVP 阶段以「冷启动 P95」为唯一硬性时延指标，对齐高层架构价值主张。
- **单二进制体积（基线已给定）**：MVP 维持 ≤ 61MB 并启动体积优化，完整版目标 ≤ 40MB（N2 / D52/D53）。属分发指标，纳入 §6.2 跟踪。
- **Token 流式呈现时延（经中间确认-方案A）**：MVP 阶段**不承诺 token 级 P50/P90/P99 SLA**，仅以冷启动 P95 ≤ 150ms（N1）为硬指标，并承诺定性「token 经 StreamEvent 逐帧直写、无全树重渲染，流式不卡顿」。该条款已通过 [中间确认-1] 用户裁决方案A 定稿，见 §7.1，不得作为对外 SLA 对外承诺。
- **并发用户数**：本地单进程单用户，并发会话数 = 1（无 Web 多租户）；单个进程内 agent loop 为单会话串行（maxSteps 上限 8）。非 QPS 意义的并发场景。
- **数据规模上限**：受宿主文件系统约束，无内置仓库大小上限；上下文窗口受 LLM Provider 限制；repo-map 类上下文压缩为完整版候选（research_report §4.1），本期不实现。

> **说明**：除冷启动 P95 与单二进制体积两项上游已给基线外，token 级时延 SLA 上游未提供基线，已按中间确认协议 §2.2（对外承诺）发起 [中间确认-1] 并获用户裁决方案A——MVP 不承诺 token 级 SLA，仅以冷启动 P95 ≤ 150ms 为硬指标 + 定性「流式不卡顿」，本条款已定稿。

### 6.3 操作与环境需求

> 浏览器 / 客户端兼容性、网络环境、设备规格、运行环境约束等。

- **客户端兼容性**：本地单二进制（Bun compile），支持 macOS / Linux / Windows 三大桌面平台；Windows 下 `runCommand` 自动使用 `cmd /c`（D34）。无浏览器兼容要求（纯终端工具）。
- **网络环境**：默认需可达 LLM Provider 端点（HTTPS SSE）；未设 Key 时 demo 完全离线；企业内网需放行 DeepSeek / OpenAI 端点；`assertSafeEndpoint` 拒绝向非 localhost 明文 http 发送 Key（D22）。`VYNTH_NET=off` 可切断工具联网（软开关）。
- **设备规格**：终端需支持 ANSI 真彩与 raw mode（TTY）以运行 TUI；无头模式仅需标准 stdout 管道。磁盘占用约 61MB（单二进制）；内存下限由 Bun 运行时与宿主决定，无额外显式下限。
- **运行环境约束**：运行期无需 Node.js / 外部 runtime（Bun 运行时内嵌于单二进制，D52）；构建期需 Bun ≥ 1.1（D2）。禁止引入无法被 Bun 打包的原生/wasm 模块（ADR-0003）。

### 6.4 安全性需求

#### 6.4.1 安全密码设置

- 产品**无账号 / 密码体系**（本地单用户，无登录），故密码强度要求 N/A。
- API Key 经环境变量 `VYNTH_API_KEY` 注入，不在代码/配置中硬编码；`assertSafeEndpoint` 拒绝向非 localhost 明文 http 发送 Key，向非 api.openai.com 端点发 Key 时打印告警（D22）。

#### 6.4.2 安全软件架构

- 模块间通信为进程内 `import()`，无对外暴露的网络端口；工具执行唯一出口为 `@vynth/sandbox`（高层架构 §5.2）。
- 外部接口仅 LLM Provider（HTTPS SSE，TLS 加密）；严禁明文 http 向非 localhost 发送凭据（D22）。
- 插件经动态 `import()` 执行任意代码，信任边界 = 宿主完整权限（D53），可读取 `VYNTH_API_KEY`；本期无插件沙箱化，须在文档与已知局限中明确（US-8 / R-01）。
- 限制外部应用系统所能获取内容：插件能力由 `-p` 显式指定路径控制，不自动加载未知插件（O6 插件市场本期不做）。

#### 6.4.3 安全设计

- 认证授权：本地无多用户认证；LLM Provider 经 API Key 认证（用户显式注入）。
- 访问控制：文件系统访问受 `safeResolve` 越界守卫约束（cwd 内 + symlink 二次校验）；网络访问受 `VYNTH_NET` 软开关约束（D34）。
- 提供资源访问的认证与授权边界：工具调用需经 `ToolRegistry`（未知工具返回 `{ok:false}`，不抛异常，D23）。

#### 6.4.4 安全开发

- 函数入口参数校验：`builtinTools` 参数经 sandbox 解析；`loadPlugin` 校验导出 `pluginName`+`activate`（D42）。
- 输入边界检查：`safeResolve` 做路径边界与符号链接二次校验（D34）；`runCommand` 校验 `networkAllowed` 与超时（D34）。
- 防高危漏洞：`assertSafeEndpoint` 防凭据泄露（D22）；不引入无法 Bun 打包的原生/wasm 模块（ADR-0003）。
- 输入输出过滤：工具结果统一 `ToolResult{ok,output,error?}` 结构化返回（D18）；禁止未授权代码（仅 `-p` 显式指定插件路径）。
- 无遗留后门：单二进制来源可追（git tag 重建 + `dist/vynth.prev` 快照，D55）。

#### 6.4.5 安全测试和部署

- 安全扫描测试：`packages/harness` 的 e2e 用例覆盖 sandbox 越界 / symlink 逃逸 / `VYNTH_NET=off` 阻断联网 / 明文 http 拒绝等（D46）。
- 安全配置基线：仅环境变量配置，无明文配置文件（ADR-0003）；无 Key 即 demo，降低凭据暴露面。
- 安全功能测试：sandbox 读/写在 cwd 内、拒绝逃逸、网络阻断等用例为必跑项（D46）。
- 上线前无高危风险：设计信任模型（软隔离）作为**已知局限**显式文档化（非未修复高危）；OS 级硬隔离以完整版 F15 兜底（方案 A 冻结）。

#### 6.4.6 数据安全

- API Key / 身份鉴别信息经环境变量注入，传输走 TLS（HTTPS SSE），不在本地明文落盘（无本地密码存储）。
- `dataDir`（默认 `~/.vynth`）仅存放本地会话/状态数据，不含凭据明文；跨进程凭据不共享（本地单用户）。

---

## 7. 待确认项与中间确认记录

### 7.1 已发起的中间确认

| 编号 | 论题 | 状态 | 影响范围 |
| --- | --- | --- | --- |
| [中间确认-1] | §6.2 Token 流式呈现时延是否承诺 P50/P90/P99 SLA（还是仅以冷启动 P95 为硬指标） | 已裁决：方案A（不承诺 token SLA，仅以冷启动 P95≤150ms 为硬指标 + 定性「流式不卡顿」） | 仅 §6.2 性能响应需求（token 时延 SLA 子项）；其余章节已先行定稿 |

### 7.2 继承自上游、本阶段不再重复发起的已裁决项

| 编号 | 决策 | 来源 | 本阶段处理 |
| --- | --- | --- | --- |
| 方案 A 沙箱推迟 | MVP 维持设计信任模型（软 `VYNTH_NET`），OS 级硬隔离推迟至完整版 F15 | 高层架构 §C.2（G3 已 [中间确认]） | 直接继承，不重复发起；US-6/US-8 明确标注已知局限 |
| X1/X2 默认 DeepSeek | 默认 `https://api.deepseek.com/v1` + `deepseek-chat`，文档向代码对齐 | 高层架构 D4（冻结） | US-7 直接采用，达成 V5 |
| X3 插件无头已实现 | `-p/--plugin` 已接入无头模式 | 高层架构 §6.1 / D51 | US-4 直接采用 |
| X4 symlink 已修复 | `safeResolve` 现做符号链接二次校验 | 高层架构 §C.2 / D34 | US-6 验收标准采用已修复行为 |

### 7.3 需后续阶段关注的待确认项（转安全/部署设计）

| 编号 | 待确认项 | 建议归属 |
| --- | --- | --- |
| U-02 | 单二进制能否压到 20–40MB（当前 61MB 含 react 残留） | 系统设计 / 部署设计 |
| U-03 | 是否引入 OS 级沙箱（方案 A 已定推迟，完整版 F15） | 安全设计 |
| U-05 | MCP 接入优先级与协议版本（2024-11-05 vs 2025-11-25） | 系统设计（完整版 F12） |

---

## 附录 A：阶段内中间确认自检报告（按协议 §2.4 在 §3/§4/§5/§6 产出后执行）

> 依据 `skills/aicoding-team-bootstrap/protocols/intermediate_confirmation.md` §2.1 + §2.3，在 §3 / §4 / §5 / §6 关键章节完成后完成自检。本附录记录所有自检结论与已触发 / 未触发确认项，供 team-lead 在 G4 审核弹窗追溯。

### A.1 自检点：§3 功能清单与高层架构 §6.3 互查

- **§2.1 判定**：未命中。功能清单（F1–F15 + N1/N2/N3）直接复用《高层架构设计》§6.3 已冻结结论，编号、优先级、MVP/完整版归属均逐行一致，无 ≥2 种合理分歧需用户裁决。
- **§2.3 反向验证 3 问**：
  - **Q1 返工成本**：若推翻本表任一功能优先级/归属，返工范围 = §3 整表 + 与高层架构 §6.3 一致性校验（约 1 表 + 跨文档一致性）；但当前为直接复用、未做改动，切换成本 ≈ 0 人月。证据：§3.1 行与高层架构 §6.3 行一一对应（F1–F15 编号、P0=P1/P2、MVP ✅/❌ 完全一致）。
  - **Q2 用户感知**：功能是否提供对用户可感知，但本表与已冻结高层架构完全一致，未改变任何用户可感知形态；故不造成新增感知偏差。证据：未新增/删减任何功能，仅转述冻结结论。
  - **Q3 与诉求一致**：一致。直接引用用户诉求「vibe coding TUI 终端编程工具，类比 Claude Code/OpenCode/Codex」+ 高层架构已冻结 F1–F11 MVP；本表未偏离。证据：§3.1「范围说明」明确「P0 ＝ MVP，与高层架构 §6.3 完全一致」。
- **结论**：未命中，不发起确认；§3 与高层架构互查通过。

### A.2 自检点：§4 角色清单与场景

- **§2.1 判定**：未命中。角色清单（R1–R5）直接复用高层架构 §2.1 五类角色（甲方决策者/最终用户 A/最终用户 B/安全合规 reviewer/插件开发者），未细分、未新增角色，无方案分歧。
- **§2.3 反向验证 3 问**：
  - **Q1 返工成本**：若推翻角色分类，返工范围 = §4.1 表（约 1 表）；但为直接复用、未改动，切换成本 ≈ 0 人月。证据：§4.1 五行与高层架构 §2.1.1–§2.1.3 逐角色对应（业务身份/主要操作/核心关注点一致）。
  - **Q2 用户感知**：角色是对用户的分类描述，未新增可感知的产品行为变化；当前与已冻结角色一致，不造成感知偏差。证据：未将角色细分为子角色（如未拆「运营管理员/合规管理员」），避免触发系统-architect 模块边界变化。
  - **Q3 与诉求一致**：一致。用户诉求未显式定义角色清单，但高层架构 §2.1 已基于诉求 + D 编号事实冻结五角色；本表逐字继承。证据：§4.1 与高层架构 §2.1 关注点对齐。
- **结论**：未命中，不发起确认。

### A.3 自检点：§5 用户旅程（US）拆分粒度

- **§2.1 判定**：未命中。8 条 US（US-1–US-8）覆盖 MVP 功能 F1–F11，未新增功能、未合并跨角色功能导致功能清单变化；拆分粒度与 §3 功能清单一致，无方案分歧。
- **§2.3 反向验证 3 问**：
  - **Q1 返工成本**：若推翻 US 拆分（如合并/拆细），返工范围 = §5 全部 US 段落（约 8 段）；但当前拆分未改变 §3 功能清单总数（仍为 F1–F15 + N1/N2/N3），切换成本 ≈ 0.5 人天（文档重排），不涉及工程实现。证据：每条 US 的「外部集成接口/功能映射」均回指 §3 既有功能项，无越界新增。
  - **Q2 用户感知**：US 是用户旅程描述，不改变功能边界或对外产品形态；用户可感知的仍是 F1–F11 既有能力。证据：US 未引入 §3 外的功能。
  - **Q3 与诉求一致**：一致。用户诉求「终端编程工具」由 F1–F11 覆盖，US 仅将其展开为可验收旅程。证据：§5 每条 US 标注覆盖的 F 编号，均在 §3 MVP 范围内。
- **结论**：未命中，不发起确认。

### A.4 自检点：§6 非功能性需求（命中 §6.2 token 时延 SLA）

- **§2.1 判定**：**命中**。§6.2 中「Token 流式呈现时延是否承诺 P50/P90/P99 SLA」存在 ≥2 种合理方案：
  - 方案 A（推荐）：MVP 不承诺 token 级 SLA，仅以冷启动 P95 ≤ 150ms 为唯一硬指标 + 定性「流式不卡顿」，避免对外部 LLM Provider 网络抖动做无法兜底的承诺。
  - 方案 B：承诺具体 SLA（如首 token P50≤800ms、P90≤2s；inter-token P90≤200ms），需绑定特定 Provider 与网络基线。
  两种方案均合理：方案 A 与「本地优先工具、时延主要取决于外部 Provider」事实一致、避免越权承诺；方案 B 与「用户体验量化」诉求一致但会绑定不可控外部因素。该决策影响下游测试基线/对外承诺（security-architect/platform-architect 不强制，但 system-architect 测试规范会引用），且用户诉求未显式要求 token 级 SLA。
- **§2.2 判定**：命中。该决策跨越「对外承诺」边界（性能指标 SLA 属对外承诺，协议 §2.2(2)），且若设定过激 SLA 日后被推翻将绑定测试/监控基线（协议 §2.2(1) 潜在不可逆）。
- **§2.3 反向验证 3 问**：
  - **Q1 返工成本**：若推翻 SLA 取值，返工范围 = §6.2 该子项 + 下游 system-architect 测试阈值；切换成本 ≈ 0.5 人天（文档 + 测试基线），未达月级，但属对外承诺绑定。证据：仅 §6.2 token 时延段 + 测试规范。
  - **Q2 用户感知**：用户/客户可感知——SLA 是性能承诺，用户会据此判断产品达标与否。证据：SLA 数值直接影响「流式不卡顿」的可量化承诺。
  - **Q3 与诉求一致**：用户诉求未显式提及 token 级时延 SLA；本决策若自行设定具体数值，属「用户诉求未显式提及但本决策改变了对外承诺」。证据：诉求原文仅要求「终端编程工具」，未给时延 SLA。
- **结论**：**命中，已发起 `[中间确认-1]`**（见 §7.1），并经用户裁决方案A（不承诺 token 级 SLA，仅以冷启动 P95 ≤ 150ms 为硬指标 + 定性「流式不卡顿」），§6.2 token 时延条款已定稿。其余 §6.1/§6.3/§6.4 沿用冻结的安全/环境模型，未触发确认，已先行定稿。

### A.5 图表生成记录

- 工具：Graphviz（`dot`），字体 PingFang SC（macOS 系统字体，支持 CJK）。
- 源文件：`/Users/zfkc/Desktop/04-AI/vynth/.workbuddy/output/pic/userstory/us_journey.dot`
- 输出文件：`us_journey.png`（1681×404）、`us_journey.svg`
- 用途：§4.3 角色 → 旅程 → 功能 映射总览，辅助 G4 审核理解 US 与功能/角色关系。
