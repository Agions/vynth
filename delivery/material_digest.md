# AICoding 架构设计 · 资料摘要

> 本文档做一件事：**精读主理人转交的全部原始资料，逐份、逐章节做出摘要**——后面任何人拿到这份摘要，都能通过章节号快速定位回原始文件的对应位置。
>
> 上游输入：主理人转交的 Vynth 项目全部原始资料（源码 `.ts`、配置 `.json`/`.yaml`、文档 `.md`）；
> 产出者：`knowledge-ingest-engineer`（知识摄入工程师 - 闻资料），经 G1 校验与人工审核通过后交付。

---

## 0. 元信息

```yaml
标题: Vynth - 资料摘要 v0.1
版本: v0.1
状态: Draft
创建日期: 2026-07-24
整理人: knowledge-ingest-engineer (wen)
审核人:
  - team-lead (主理人)

原始资料清单:
  - D0 用户原始诉求: 主理人转交，vibe coding TUI 终端编程工具，类比 Claude Code/OpenCode/Codex
  - D1 README.md: 项目总览与架构声明
  - D2 package.json: 根构建/运行配置
  - D3 pnpm-workspace.yaml: pnpm 工作区声明
  - D4 tsconfig.base.json: TS 基础编译配置
  - D5 biome.json: 格式化与 lint 配置
  - D6 turbo.json: 任务编排配置
  - D7 .gitignore: 忽略项
  - D8-D10 apps/cli: CLI 入口包（package.json/tsconfig/src/main.ts）
  - D11-D18 packages/core: 共享基础（config/errors/events/logger/types/index）
  - D19-D24 packages/engine: LLM 客户端 + 工具系统 + agent 循环
  - D25-D30 packages/tui: 自研轻量 ANSI 渲染器 + 主题 + 流式逃生舱
  - D31-D34 packages/sandbox: fs 路径越界守卫 + 命令执行
  - D35-D38 packages/mcp: MCP stdio JSON-RPC 客户端
  - D39-D43 packages/plugins: 插件加载/生命周期 + 示例
  - D44-D47 packages/harness: e2e 集成测试 + fixture
  - D48-D55 docs/*: 架构/API/ADR/指南/发布说明/回滚文档
  - D56 scripts/mock-llm.ts: 本地 mock LLM 服务
```

| 版本 | 日期 | 作者 | 变更内容 |
| --- | --- | --- | --- |
| v0.1 | 2026-07-24 | `knowledge-ingest-engineer` | 初稿（G1 阶段，逐份精读 56 份资料） |

---

## 1. 资料清单

> 列出全部原始资料，每份标注解析状态。本项目资料均为文本型（源码 / markdown / json / yaml），无需调用 docx / pdf / pptx / xlsx 等二进制 Skill，统一以直读方式解析。

| 编号 | 文件名 | 类型 | 来源 | 解析状态 | 说明 |
| --- | --- | --- | --- | --- | --- |
| D0 | 用户原始诉求（主理人转交） | text | 主理人 | 已解析 | 上游诉求文本，作为本摘要第 0 份资料纳入 |
| D1 | `README.md` | md | 项目根 | 已解析 | — |
| D2 | `package.json`（根） | json | 项目根 | 已解析 | — |
| D3 | `pnpm-workspace.yaml` | yaml | 项目根 | 已解析 | — |
| D4 | `tsconfig.base.json` | json | 项目根 | 已解析 | — |
| D5 | `biome.json` | json | 项目根 | 已解析 | — |
| D6 | `turbo.json` | json | 项目根 | 已解析 | — |
| D7 | `.gitignore` | text | 项目根 | 已解析 | — |
| D8 | `apps/cli/package.json` | json | apps/cli | 已解析 | — |
| D9 | `apps/cli/tsconfig.json` | json | apps/cli | 已解析 | — |
| D10 | `apps/cli/src/main.ts` | ts | apps/cli | 已解析 | CLI 入口 |
| D11 | `packages/core/package.json` | json | packages/core | 已解析 | — |
| D12 | `packages/core/tsconfig.json` | json | packages/core | 已解析 | — |
| D13 | `packages/core/src/index.ts` | ts | packages/core | 已解析 | — |
| D14 | `packages/core/src/config.ts` | ts | packages/core | 已解析 | 配置默认值（与多处文档冲突，见 §3 X1/X2） |
| D15 | `packages/core/src/errors.ts` | ts | packages/core | 已解析 | — |
| D16 | `packages/core/src/events.ts` | ts | packages/core | 已解析 | — |
| D17 | `packages/core/src/logger.ts` | ts | packages/core | 已解析 | — |
| D18 | `packages/core/src/types.ts` | ts | packages/core | 已解析 | — |
| D19 | `packages/engine/package.json` | json | packages/engine | 已解析 | — |
| D20 | `packages/engine/tsconfig.json` | json | packages/engine | 已解析 | — |
| D21 | `packages/engine/src/index.ts` | ts | packages/engine | 已解析 | — |
| D22 | `packages/engine/src/llm.ts` | ts | packages/engine | 已解析 | — |
| D23 | `packages/engine/src/tools.ts` | ts | packages/engine | 已解析 | — |
| D24 | `packages/engine/src/agent-loop.ts` | ts | packages/engine | 已解析 | — |
| D25 | `packages/tui/package.json` | json | packages/tui | 已解析 | — |
| D26 | `packages/tui/tsconfig.json` | json | packages/tui | 已解析 | — |
| D27 | `packages/tui/src/index.ts` | ts | packages/tui | 已解析 | — |
| D28 | `packages/tui/src/tui.ts` | ts | packages/tui | 已解析 | — |
| D29 | `packages/tui/src/stream-escape-hatch.ts` | ts | packages/tui | 已解析 | — |
| D30 | `packages/tui/src/theme.ts` | ts | packages/tui | 已解析 | — |
| D31 | `packages/sandbox/package.json` | json | packages/sandbox | 已解析 | — |
| D32 | `packages/sandbox/tsconfig.json` | json | packages/sandbox | 已解析 | — |
| D33 | `packages/sandbox/src/index.ts` | ts | packages/sandbox | 已解析 | — |
| D34 | `packages/sandbox/src/sandbox.ts` | ts | packages/sandbox | 已解析 | 符号链接校验（与 D1 冲突，见 §3 X4） |
| D35 | `packages/mcp/package.json` | json | packages/mcp | 已解析 | — |
| D36 | `packages/mcp/tsconfig.json` | json | packages/mcp | 已解析 | — |
| D37 | `packages/mcp/src/index.ts` | ts | packages/mcp | 已解析 | — |
| D38 | `packages/mcp/src/mcp-client.ts` | ts | packages/mcp | 已解析 | 尚未并入 CLI（见 §3 状态说明） |
| D39 | `packages/plugins/package.json` | json | packages/plugins | 已解析 | — |
| D40 | `packages/plugins/tsconfig.json` | json | packages/plugins | 已解析 | — |
| D41 | `packages/plugins/src/index.ts` | ts | packages/plugins | 已解析 | — |
| D42 | `packages/plugins/src/loader.ts` | ts | packages/plugins | 已解析 | — |
| D43 | `packages/plugins/examples/hello-plugin.ts` | ts | packages/plugins | 已解析 | 示例插件 |
| D44 | `packages/harness/package.json` | json | packages/harness | 已解析 | — |
| D45 | `packages/harness/tsconfig.json` | json | packages/harness | 已解析 | — |
| D46 | `packages/harness/src/e2e.test.ts` | ts | packages/harness | 已解析 | — |
| D47 | `packages/harness/src/fixtures/sample-plugin.ts` | ts | packages/harness | 已解析 | — |
| D48 | `docs/index.md` | md | docs | 已解析 | — |
| D49 | `docs/architecture/index.md` | md | docs/architecture | 已解析 | — |
| D50 | `docs/architecture/packages.md` | md | docs/architecture | 已解析 | 插件 CLI 加载状态与外部文档冲突，见 §3 X3 |
| D51 | `docs/api/overview.md` | md | docs/api | 已解析 | — |
| D52 | `docs/adr/0003-pure-typescript-build.md` | md | docs/adr | 已解析 | 架构决策记录 |
| D53 | `docs/guide/getting-started.md` | md | docs/guide | 已解析 | — |
| D54 | `docs/release-notes-v0.1.0.md` | md | docs | 已解析 | — |
| D55 | `docs/rollback.md` | md | docs | 已解析 | — |
| D56 | `scripts/mock-llm.ts` | ts | scripts | 已解析 | — |

**类型枚举说明**：本批次资料均为文本型。源码标注 `ts`，文档标注 `md`，JSON/YAML 配置分别标注 `json`/`yaml`，忽略规则标注 `text`。未涉及 `docx`/`pdf`/`pptx`/`xlsx`，故未调用对应二进制解析 Skill。

---

## 2. 资料内容摘要

> 逐份文档按自身章节/逻辑结构做摘要。每条摘要标注章节号（`D编号，§章节`），后面任何人想核实某个点，直接定位回原文对应位置即可。

### D0：用户原始诉求（主理人转交）

> 上游一句话诉求，作为第 0 份资料纳入整理 — 来源：主理人（team-lead）转交

| 章节 | 内容摘要 |
| --- | --- |
| D0，§诉求 | 项目定位：vibe coding TUI 终端编程工具；类比对象为 Claude Code、OpenCode、Codex 等终端编程工具（直接引用 / 综合归纳，无推断） |

### D1：`README.md`

> 项目总览与架构声明，面向使用者的快速说明 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D1，§标题/定位 | 第 1–5 行：Vynth = 「你 terminal 里的代码合成器」（AI-Native Coding Terminal）；终端 TUI 的 Vibe Coding 工具，支持 **Plan** / **Vibe** 双模式，把自然语言「合成」成代码 |
| D1，§架构（纯 TypeScript 全量构建） | 第 7–24 行：运行时 = Bun + `bun build --compile` 单二进制；TUI = 自研轻量 ANSI 渲染器（非 ink）+ 流式直写逃生舱；结构 = pnpm workspace + turbo，包以 `@vynth/*` 组织；列出 packages/（core/engine/tui/sandbox/mcp/plugins/harness）与 apps/cli；sandbox 注释「fs 路径越界守卫（可被符号链接绕过）；run_shell 以宿主权限运行、无隔离」 |
| D1，§快速开始 | 第 26–34 行：`bun install`（或 `pnpm install`）→ `bun run compile` → `./dist/vynth --help` / `-g "目标"` / 无参启动 TUI；无 `VYNTH_API_KEY` 时自动进入 **demo（echo）provider** 可离线体验 |
| D1，§配置 | 第 38–46 行：仅通过环境变量配置，不读取任何配置文件；列出 `VYNTH_LLM_BASE_URL`（默认 `https://api.openai.com/v1`）、`VYNTH_API_KEY`、`VYNTH_MODEL`（默认 `gpt-4o-mini`）、`VYNTH_MODE`（默认 `vibe`）——**未列出** `VYNTH_THEME` / `VYNTH_NET` / `VYNTH_DATA_DIR`（与代码及 docs 不一致，见 §3 X6） |

### D2：`package.json`（根）

> 根构建/运行配置 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D2，§name/version | `name: vynth`，`version: 0.1.0`，`private: true`，`type: module`，`license: MIT` |
| D2，§workspaces | `"workspaces": ["packages/*", "apps/*"]`——此为 npm/bun 风格 workspaces 字段（与 pnpm-workspace.yaml 并存，见 §3 X5） |
| D2，§scripts | `compile` = `bun build --compile --target=bun --minify apps/cli/src/main.ts --outfile dist/vynth`；`build` = `bun run compile`；`dev` = `bun run apps/cli/src/main.ts`；`test` = `bun test packages`；`lint` = `biome check .`；`fmt` = `biome format --write .` |
| D2，§devDependencies | `@biomejs/biome ^1.9.4`，`typescript ^5.6.3` |
| D2，§engines | `bun: ">=1.1"` |

### D3：`pnpm-workspace.yaml`

> pnpm 工作区声明 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D3，§packages | `packages:` 下声明 `- "packages/*"` 与 `- "apps/*"`；这是 pnpm 识别工作区的方式（与 D2 的 package.json `workspaces` 字段并存，见 §3 X5） |

### D4：`tsconfig.base.json`

> TypeScript 基础编译配置 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D4，§compilerOptions | `target: ES2022`，`module: ESNext`，`moduleResolution: Bundler`，`lib: [ES2022, DOM]`，`jsx: react-jsx`，`strict: true`，`noEmit: true`，`esModuleInterop: true`，`skipLibCheck: true`，`forceConsistentCasingInFileNames: true`，`verbatimModuleSyntax: false`，`resolveJsonModule: true`，`allowImportingTsExtensions: false`，`types: ["node", "react", "react-dom"]` |

### D5：`biome.json`

> 格式化与 lint 配置 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D5，§organizeImports/§formatter | `organizeImports.enabled: true`；formatter 启用，缩进 space/2，行宽 100，换行 lf |
| D5，§javascript.formatter | `semicolons: always`，`quoteStyle: single`，`trailingCommas: none` |
| D5，§linter.rules | `recommended: true`；`correctness.useImportExtensions: off`；`style.useNodejsImportProtocol: error`、`style.noDefaultExport: error`；`suspicious.noExplicitAny: error`、`noConsoleLog: off` |
| D5，§files | `ignore: ["dist", "node_modules", "**/*.md"]` |

### D6：`turbo.json`

> 任务编排配置 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D6，§tasks | `build`：`dependsOn: ["^build"]`，`outputs: ["dist/**"]`；`test`：`dependsOn: ["^build"]`；`lint`：无依赖 |

### D7：`.gitignore`

> 忽略项 — 来源：项目根

| 章节 | 内容摘要 |
| --- | --- |
| D7，§ignore | `node_modules/` `dist/` `*.log` `.env*` `.DS_Store` `.vynth/` |

### D8：`apps/cli/package.json`

> CLI 应用包配置 — 来源：apps/cli

| 章节 | 内容摘要 |
| --- | --- |
| D8，§name/bin | `name: @vynth/cli`，`bin: { "vynth": "src/main.ts" }` |
| D8，§dependencies | `@vynth/core` / `@vynth/engine` / `@vynth/tui` / `@vynth/plugins`（均 `workspace:*`） |
| D8，§scripts | `build` = `bun build --compile --target=bun src/main.ts --outfile ../../dist/vynth` |

### D9：`apps/cli/tsconfig.json`

> CLI 包编译配置 — 来源：apps/cli

| 章节 | 内容摘要 |
| --- | --- |
| D9，§compilerOptions | 继承 `../../tsconfig.base.json`；`rootDir: src`，`outDir: dist`，`jsx: react-jsx`；`include: ["src"]` |

### D10：`apps/cli/src/main.ts`

> CLI 入口源码 — 来源：apps/cli

| 章节 | 内容摘要 |
| --- | --- |
| D10，§imports | 从 `@vynth/core` 引入 `Mode`、`loadConfig`；从 `@vynth/engine` 引入 `builtinTools`、`createProvider`、`runAgent`；从 `@vynth/plugins` 引入 `loadPlugin`；从 `@vynth/tui` 引入 `startTui` |
| D10，§VERSION | `const VERSION = '0.1.0'` |
| D10，§parseArgs | 解析 `-v/--version`、`-h/--help`、`-g/--goal`、`-m/--mode`、`-p/--plugin` |
| D10，§printHelp | 打印用法 + 环境变量：`VYNTH_API_KEY` / `VYNTH_MODEL` / `VYNTH_LLM_BASE_URL` / `VYNTH_MODE` / `VYNTH_THEME`（mocha｜latte）（注：help 文本未含 `VYNTH_NET` / `VYNTH_DATA_DIR`，见 §3 X6） |
| D10，§runHeadless | `loadConfig()` → `createProvider(config)` → `builtinTools(config.sandbox.cwd, { networkAllowed: config.sandbox.networkAllowed })`；若 `pluginPath` 则 `loadPlugin(abs)` + `plugin.activate(tools)` 后流式直写 stdout；`runAgent` 循环 `token`/`tool` 事件 |
| D10，§main | `parseArgs` → 处理 version/help → `loadConfig({ mode })` → 有 goal 走 `runHeadless` → 非 TTY 时 `process.exit(2)` 提示改用无头 → 否则 `startTui(config)` |

### D11：`packages/core/package.json`

> core 包配置 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D11，§name/main | `name: @vynth/core`，`main/types/exports` 均指向 `src/index.ts`；`build` = `echo` 占位（workspace 内消费） |

### D12：`packages/core/tsconfig.json`

> core 包编译配置 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D12，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D13：`packages/core/src/index.ts`

> core 统一导出 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D13，§exports | 再导出 `./types`、`./errors`、`./events`、`./logger`、`./config` |

### D14：`packages/core/src/config.ts`

> 配置加载（实际生效代码，默认值与多处文档冲突，见 §3 X1/X2） — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D14，§loadConfig(overrides?) | 模式：`VYNTH_MODE` 或 `overrides.mode`，非 plan/vibe 则默认 `vibe`；网络：`VYNTH_NET`，默认开启（`0/off/false/no` 关闭） |
| D14，§默认值 | `llmBaseUrl` = `https://api.deepseek.com/v1`（**不是** openai.com/v1）；`apiKey` = `''`；`model` = `deepseek-chat`（**不是** gpt-4o-mini）；`theme` = `VYNTH_THEME==='latte' ? 'latte' : 'mocha'`；`sandbox.networkAllowed` 由 `VYNTH_NET` 决定；`sandbox.cwd` = `process.cwd()`；`dataDir` = `VYNTH_DATA_DIR` 或 `~/.vynth` |
| D14，§读取方式 | 仅读 `process.env`，不读配置文件（与 README/ADR 描述一致） |

### D15：`packages/core/src/errors.ts`

> 错误体系 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D15，§VynthError | 基类，带 `code` 字段，`name: 'VynthError'` |
| D15，§子类 | `ConfigError`(code `config`) / `LlmError`(`llm`) / `ToolError`(`tool`) / `SandboxError`(`sandbox`) / `McpError`(`mcp`) / `PluginError`(`plugin`)，各自携带对应 code |

### D16：`packages/core/src/events.ts`

> 类型安全事件总线 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D16，§Emitter（泛型 M） | `on(key, fn)` 返回退订函数（`set.delete`）；`emit(key, payload)` 遍历订阅者调用；内部用 `Map`（键为 M 的键、值为监听者集合） |

### D17：`packages/core/src/logger.ts`

> 分级日志 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D17，§LogLevel/§setLogLevel | 级别 `debug/info/warn/error`；`setLogLevel` 设置当前级别 |
| D17，§log | `log(level, message, meta?)`：低于当前级别则丢弃；error 与常规均走 `console.error`（注意：info/warn 也走 console.error，非 stdout） |

### D18：`packages/core/src/types.ts`

> 全局类型定义 — 来源：packages/core

| 章节 | 内容摘要 |
| --- | --- |
| D18，§Mode | `type Mode = 'plan' \| 'vibe'` |
| D18，§ChatMessage | `role: system/user/assistant/tool`，`content: string`，可选 `name` |
| D18，§ToolParam/ToolResult/ToolDef/ToolCall | `ToolParam{name,type,description,required}`；`ToolResult{ok,output,error?}`；`ToolDef{name,description,parameters,run}`；`ToolCall{id,name,args}` |
| D18，§StreamEvent | 联合类型：`{type:'token',text}` / `{type:'tool',call:ToolCall}` / `{type:'done',usage?}` |
| D18，§VynthConfig | `mode/llmBaseUrl/apiKey/model/theme('mocha'\|'latte')/sandbox{networkAllowed,cwd}/dataDir` |

### D19：`packages/engine/package.json`

> engine 包配置 — 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D19，§name/deps | `name: @vynth/engine`；依赖 `@vynth/core`、`@vynth/sandbox`（`workspace:*`）；`build` = echo |

### D20：`packages/engine/tsconfig.json`

> engine 包编译配置 — 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D20，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D21：`packages/engine/src/index.ts`

> engine 统一导出 — 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D21，§exports | 再导出 `./llm`、`./tools`、`./agent-loop` |

### D22：`packages/engine/src/llm.ts`

> LLM 客户端（OpenAI 兼容 SSE）— 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D22，§LLMProvider | 接口：`chat(messages, tools)` 返回 `StreamEvent` 的异步可迭代（AsyncIterable） |
| D22，§createProvider | `apiKey` 为空 → `EchoProvider`；否则 `OpenAiProvider` |
| D22，§OpenAiProvider | 构造时 `assertSafeEndpoint`；`fetch POST {baseUrl}/chat/completions`，逐行解析 `data:` SSE 帧，累加 `tool_calls`（按 index 聚合），`finish_reason` 时取 usage，输出 `token`/`tool`/`done` 事件 |
| D22，§assertSafeEndpoint | 拒绝向非 localhost 的明文 `http` 发送 API Key；向非 `api.openai.com` 的端点发 key 时打印告警 |
| D22，§EchoProvider | goal 含 `demo-tool` 且存在工具时调用首个工具并填示例参数；否则回显中文「（demo）收到目标：…」 |
| D22，§toJsonProps | `ToolDef` → OpenAI `function.parameters`（`type/description/required`） |

### D23：`packages/engine/src/tools.ts`

> 工具注册表与内置工具 — 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D23，§ToolRegistry | `register`（重名抛 `ToolError`）/`get`/`list`/`run`（未知工具返回 `{ok:false}`，不抛异常） |
| D23，§builtinTools | `builtinTools(cwd, { networkAllowed? })`：注册 `read_file`（→`sandbox.readText`）、`write_file`（→`sandbox.writeText`）、`run_shell`（→`sandbox.runCommand`，透传 `cwd` 与 `networkAllowed`） |

### D24：`packages/engine/src/agent-loop.ts`

> agent 循环 — 来源：packages/engine

| 章节 | 内容摘要 |
| --- | --- |
| D24，§AgentOpts | `{ provider, tools, system?, maxSteps? }` |
| D24，§DEFAULT_SYSTEM | 中文系统提示：「你是 Vynth，一个终端内的 AI 编程助手…」 |
| D24，§runAgent | `StreamEvent` 异步生成器（AsyncGenerator）：组装 `messages=[system,user(goal)]`；`toolDefs=tools.list()`；`maxSteps` 默认 **8**；循环 `provider.chat` 收集 token/pendingTool；有 pendingTool 则 `yield tool` → `tools.run` → 回填 `[assistant, tool]` |

### D25：`packages/tui/package.json`

> tui 包配置 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D25，§name/deps | `name: @vynth/tui`；依赖 `@vynth/core`、`@vynth/engine`、`ansi-escapes ^6.2.0`；`build` = echo |

### D26：`packages/tui/tsconfig.json`

> tui 包编译配置 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D26，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`，`jsx: react-jsx`；`include: ["src"]` |

### D27：`packages/tui/src/index.ts`

> tui 统一导出 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D27，§exports | 导出 `startTui`、`StreamArea`、`palette`/`fg`/`bg`/`reset` |

### D28：`packages/tui/src/tui.ts`

> 轻量 ANSI TUI 主逻辑 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D28，§注释 | 顶部注释明确：轻量 ANSI TUI（无外部布局引擎），保证 `bun build --compile` 单二进制不依赖 `yoga.wasm` 等外部资源；高频 token 更新走 StreamArea 直写规避重渲染 |
| D28，§startTui | `palette(theme)` → `createProvider` → `builtinTools(cwd,{networkAllowed})`；`readline` + keypress + raw mode；`StreamArea` 直写；`draw()` 全屏重绘 `\x1b[2J\x1b[H`；keypress 处理 ctrl-c/d 退出（恢复 raw mode） |
| D28，§submit | 触发 `runAgent`，流式 `token` 写入 live 区，`tool` 回显至 history；结束后 `cleanup` 恢复 raw mode 并 close readline |

### D29：`packages/tui/src/stream-escape-hatch.ts`

> 流式逃生舱 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D29，§StreamArea | 用 `ansi-escapes` 的 `cursorLeft`+`clearLine` 行内直写，绕过 ink 的 React reconciliation，避免每个 token 触发全树重渲染；`update(text)` / `clear()` 维护 `lastLen` |

### D30：`packages/tui/src/theme.ts`

> Catppuccin 主题与 ANSI 调色 — 来源：packages/tui

| 章节 | 内容摘要 |
| --- | --- |
| D30，§Palette | 接口含 base/mantle/crust/text/subtext/mauve/lavender/teal/green/red/yellow/blue |
| D30，§catppuccin | `mocha` 与 `latte` 两套具体 hex 值（如 mocha base `#1e1e2e`、mauve `#cba6f7`） |
| D30，§helpers | `palette(theme)`、`reset`、`fg(hex)`/`bg(hex)` 真彩 ANSI 转义、`hexToRgb` |

### D31：`packages/sandbox/package.json`

> sandbox 包配置 — 来源：packages/sandbox

| 章节 | 内容摘要 |
| --- | --- |
| D31，§name/deps | `name: @vynth/sandbox`；依赖 `@vynth/core`；`build` = echo |

### D32：`packages/sandbox/tsconfig.json`

> sandbox 包编译配置 — 来源：packages/sandbox

| 章节 | 内容摘要 |
| --- | --- |
| D32，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D33：`packages/sandbox/src/index.ts`

> sandbox 统一导出 — 来源：packages/sandbox

| 章节 | 内容摘要 |
| --- | --- |
| D33，§exports | 再导出 `./sandbox` |

### D34：`packages/sandbox/src/sandbox.ts`

> fs 越界守卫与命令执行（与 README 描述冲突，见 §3 X4）— 来源：packages/sandbox

| 章节 | 内容摘要 |
| --- | --- |
| D34，§safeResolve | 先 `resolve(cwd, target)`，必须落在 `cwd` 内否则抛 `SandboxError`；再 `realpathSync` 解析符号链接做**二次校验**，cwd 内 symlink 指向沙箱外 → 抛 `SandboxError`（即「符号链接逃逸已在代码层修复」） |
| D34，§readText/§writeText | 经 `safeResolve` 后 `fs` 读写，返回 `ToolResult` |
| D34，§runCommand | 签名 `{ cwd, networkAllowed?, timeoutMs? }`；`networkAllowed` falsy → 直接 `{ok:false, error:'network blocked by sandbox policy'}`；`spawn sh -c`（win32 用 `cmd /c`），默认超时 **30s**（SIGKILL），合并 stdout/stderr，按 exit code 判定 `ok` |
| D34，§errMsg | 统一错误转字符串 |

### D35：`packages/mcp/package.json`

> mcp 包配置 — 来源：packages/mcp

| 章节 | 内容摘要 |
| --- | --- |
| D35，§name/deps | `name: @vynth/mcp`；依赖 `@vynth/core`；`build` = echo |

### D36：`packages/mcp/tsconfig.json`

> mcp 包编译配置 — 来源：packages/mcp

| 章节 | 内容摘要 |
| --- | --- |
| D36，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D37：`packages/mcp/src/index.ts`

> mcp 统一导出 — 来源：packages/mcp

| 章节 | 内容摘要 |
| --- | --- |
| D37，§exports | 再导出 `./mcp-client` |

### D38：`packages/mcp/src/mcp-client.ts`

> MCP stdio JSON-RPC 客户端（尚未并入 CLI，见 §3 状态说明）— 来源：packages/mcp

| 章节 | 内容摘要 |
| --- | --- |
| D38，§JsonRpc | `JsonRpcReq`（`jsonrpc/id/method/params`）、`JsonRpcRes`（`jsonrpc/id/result?/error?`）接口 |
| D38，§McpClient.connect | `spawn` 子进程（stdio），发 `initialize`（`protocolVersion: '2024-11-05'`，`clientInfo: {name:'vynth', version:'0.1.0'}`）→ `tools/list` 填充 `this.tools` |
| D38，§callTool | 发 `tools/call`，解析 `content[].text` 合并为 `ToolResult` |
| D38，§rpc/§onData | 按行解析 JSON-RPC，按 `id` 匹配 pending；`close()` kill 子进程 |

### D39：`packages/plugins/package.json`

> plugins 包配置 — 来源：packages/plugins

| 章节 | 内容摘要 |
| --- | --- |
| D39，§name/deps | `name: @vynth/plugins`；依赖 `@vynth/core`、`@vynth/engine`；`build` = echo |

### D40：`packages/plugins/tsconfig.json`

> plugins 包编译配置 — 来源：packages/plugins

| 章节 | 内容摘要 |
| --- | --- |
| D40，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D41：`packages/plugins/src/index.ts`

> plugins 统一导出 — 来源：packages/plugins

| 章节 | 内容摘要 |
| --- | --- |
| D41，§exports | 再导出 `./loader` |

### D42：`packages/plugins/src/loader.ts`

> 插件加载与生命周期 — 来源：packages/plugins

| 章节 | 内容摘要 |
| --- | --- |
| D42，§Plugin/§PluginModule | `Plugin{ name, activate(reg) }`；`PluginModule{ pluginName, activate(reg) }` |
| D42，§loadPlugin | `import(entryPath)` 动态加载，要求导出 `pluginName` 与 `activate`，否则抛 `PluginError` |
| D42，§loadAll | `loadAll(entries, reg)` 批量加载并 `activate` |
| D42，§errMsg | 统一错误转字符串 |

### D43：`packages/plugins/examples/hello-plugin.ts`

> 示例插件 — 来源：packages/plugins

| 章节 | 内容摘要 |
| --- | --- |
| D43，§pluginName/§activate | `pluginName='hello-plugin'`；`activate(reg)` 注册 `hello` 工具（`name` 参数）；加载方式：`vynth -g "用 hello 工具向世界问好" -p packages/plugins/examples/hello-plugin.ts` |

### D44：`packages/harness/package.json`

> harness 包配置 — 来源：packages/harness

| 章节 | 内容摘要 |
| --- | --- |
| D44，§name/scripts | `name: @vynth/harness`，`private: true`；依赖 `core/engine/plugins`；`scripts.test = bun test` |

### D45：`packages/harness/tsconfig.json`

> harness 包编译配置 — 来源：packages/harness

| 章节 | 内容摘要 |
| --- | --- |
| D45，§compilerOptions | 继承 base；`rootDir: src`，`outDir: dist`；`include: ["src"]` |

### D46：`packages/harness/src/e2e.test.ts`

> e2e 集成测试 — 来源：packages/harness

| 章节 | 内容摘要 |
| --- | --- |
| D46，§MockProvider | 自实现 `LLMProvider`，首轮调用首个工具，验证流式 token + 工具调用全链路 |
| D46，§用例 | `agent streams tokens and runs a tool`；`OpenAiProvider parses SSE tokens and tool calls`；`plugin activates and registers a tool`；`demo EchoProvider streams tokens when no API key`；`sandbox read_file reads within cwd and rejects escape`；`sandbox rejects symlink escape`；`VYNTH_NET=off blocks run_shell networking`；`OpenAiProvider refuses plaintext http to non-local endpoint` |

### D47：`packages/harness/src/fixtures/sample-plugin.ts`

> 测试用 fixture 插件 — 来源：packages/harness

| 章节 | 内容摘要 |
| --- | --- |
| D47，§pluginName/§activate | `pluginName='sample-plugin'`；`activate(reg)` 注册 `sample_tool`（`x: number`） |

### D48：`docs/index.md`

> 文档总入口 — 来源：docs

| 章节 | 内容摘要 |
| --- | --- |
| D48，§定位/§为什么是 Vynth | Vynth = 「你 terminal 里的代码合成器」，AI-Native Coding Terminal，Plan/Vibe 双模式，纯 TS 单二进制；卖点：terminal 原生 / 单二进制 / 轻量 TUI / demo 即开 / 可扩展 |
| D48，§30秒上手 | `bun install` → `bun run compile` → `./dist/vynth -g`；`export VYNTH_API_KEY` 接入真实 LLM |
| D48，§文档导航/§命令速览 | 链接表；`vynth` / `-g` / `-m` / `--version` / `--help` |
| D48，§配置环境变量 | 列出**全部 7 个**：`VYNTH_API_KEY`/`VYNTH_MODEL`/`VYNTH_LLM_BASE_URL`/`VYNTH_MODE`/`VYNTH_THEME`/`VYNTH_NET`/`VYNTH_DATA_DIR`（比 README §配置 完整，见 §3 X6） |

### D49：`docs/architecture/index.md`

> 架构总览 — 来源：docs/architecture

| 章节 | 内容摘要 |
| --- | --- |
| D49，§一句话架构 | goal → agent-loop → LLM 流式补全（OpenAI 兼容 SSE）→ tool call → sandbox → TUI/无头流式渲染；由 `bun build --compile` 打包为单二进制 |
| D49，§运行形态（双模式） | TUI（需 TTY）/ Headless（`-g`）共用同一条 agent-loop，区别仅在渲染层 |
| D49，§Package 地图 | 8 包职责与依赖表（core/engine/tui/sandbox/mcp/plugins/harness/cli） |
| D49，§实现状态 | **plugins 已通过 `-p/--plugin`（无头 `-g`）接入 CLI**；**MCP 接入仍在路线图**，`McpClient` 已就绪但未并入 CLI 的 agent 工具集 |
| D49，§数据流（端到端） | 流程图：入口 → agent-loop → LLM/Sandbox → 渲染层 |
| D49，§关键不变量 | StreamEvent 是唯一跨层协议；ToolResult 统一 `{ok,output,error?}`；sandbox 是工具执行唯一出口；无 key 即 demo |
| D49，§构建与分发 | pnpm workspace + turbo；tsconfig.base（ES2022/strict/Bundler）；`bun test` |

### D50：`docs/architecture/packages.md`

> 各 `@vynth/*` 包职责详解 — 来源：docs/architecture

| 章节 | 内容摘要 |
| --- | --- |
| D50，§core | 各文件职责表；配置默认值写明 `llmBaseUrl=https://api.openai.com/v1`、`model=gpt-4o-mini`、`theme=mocha`、`sandbox.networkAllowed = VYNTH_NET !== '0'`、`cwd=process.cwd()`、`dataDir=~/.vynth`（与 D14 代码默认值冲突，见 §3 X1/X2） |
| D50，§engine | llm/tools/agent-loop 职责；对外契约；demo 行为说明 |
| D50，§tui | 非 ink；各文件职责；仅当 `isTTY` 进入 raw mode |
| D50，§sandbox | `safeResolve`/`readText`/`writeText`/`runCommand`；不变量：工具不得绕过 sandbox；网络默认开启（`VYNTH_NET !== '0'` 关闭） |
| D50，§mcp | `McpClient` 职责；实现状态：已可用但未在 CLI 接入 |
| D50，§plugins | `loader` 职责与契约；实现状态：**「CLI 的 `--plugin 路径` 加载入口由插件加载工作流补齐」**（与 D51、D10 冲突，见 §3 X3） |
| D50，§harness | `e2e.test.ts` 职责；`bun test` |
| D50，§cli | `parseArgs`/`printHelp`/`runHeadless`/`main`；`bin: vynth`；`--version` 输出 `0.1.0` |

### D51：`docs/api/overview.md`

> CLI 对外接口总览 — 来源：docs/api

| 章节 | 内容摘要 |
| --- | --- |
| D51，§子命令/参数表 | `-g/--goal`、`-m/--mode`、`--plugin`、`-v/--version`、`-h/--help`、无参启动 TUI |
| D51，§环境变量 | 7 变量表：`VYNTH_MODEL` 默认 `gpt-4o-mini`、`VYNTH_LLM_BASE_URL` 默认 `https://api.openai.com/v1`、`VYNTH_THEME` mocha/latte、`VYNTH_NET`、`VYNTH_DATA_DIR`；注明仅经环境变量注入 |
| D51，§退出码 | `0` 正常 / `2` 用法错误 / 非 0 运行期错误 |
| D51，§实现状态 | **`--plugin 路径`（CLI 加载）：✅ 已实现**（main.ts 的 `parseArgs` 解析 `-p/--plugin`，`runHeadless` 经 `loadPlugin`+`activate`）；退出码 `2` ✅ 已实现；`mcp` 接入未并入 CLI |

### D52：`docs/adr/0003-pure-typescript-build.md`

> 架构决策记录：纯 TypeScript 全量构建 — 来源：docs/adr

| 章节 | 内容摘要 |
| --- | --- |
| D52，§状态/§背景 | 状态 Accepted；由「Rust 混合架构」翻转为「纯 TS 全量构建」；动因：生态成熟度 / 招人成本 / 迭代速度 / ink 放弃 yoga.wasm 后 Rust 护城河收窄 |
| D52，§决策 | 运行时 Bun（`engines.bun >= 1.1`），`bun build --compile` 单二进制；包管理 pnpm workspace + turbo；`@vynth/*` 包组织；轻量 ANSI 渲染（明确不用 ink）；仅环境变量配置 |
| D52，§后果 | 正面：单语言/分发简单/招人面宽/冷启动 50–150ms；代价：单二进制体积目标 20–40MB（当前约 **61MB**，含 react 残留）；依赖 Bun 演进；放弃 Rust；V8 GC 偶发 5ms 以内停顿；TUI 自维护 |
| D52，§兼容性约束 | 禁止引入无法被 bun 打包的原生/wasm 模块 |
| D52，§回退 | 出现 Bun 不支持原生模块 / 跨平台受阻 / 性能缺口 → 回退 Node + esbuild（保留全部 TS 源码，仅换打包/分发层） |

### D53：`docs/guide/getting-started.md`

> 快速开始指南 — 来源：docs/guide

| 章节 | 内容摘要 |
| --- | --- |
| D53，§安装依赖/§编译单二进制 | `bun install`（或 pnpm）；Bun>=1.1；`bun run compile`；体积约 **61MB** 目标 20–40MB；冷启动 50–150ms |
| D53，§无 key 先跑 demo/§接入真实 LLM | `-g` demo；接入示例环境变量（`VYNTH_MODEL` 写作 gpt-4o-mini 但注释「默认已指向 DeepSeek」，`VYNTH_LLM_BASE_URL` 写作 openai.com/v1 但注释「默认已指向 DeepSeek」） |
| D53，§启动交互 TUI | `./dist/vynth` 需真实 TTY |
| D53，§加载插件与信任边界 | 动态 `import()` 执行任意代码；**信任边界警告**：插件宿主完整权限、可读取 `VYNTH_API_KEY`；`run_shell` 宿主权限无隔离；`VYNTH_NET='0'` 软网关非硬边界 |
| D53，§环境变量速查 | 7 变量表：`VYNTH_MODEL` 默认 **`deepseek-chat`**、`VYNTH_LLM_BASE_URL` 默认 **`https://api.deepseek.com/v1`**（与 D51/D50/D1 冲突，见 §3 X1/X2） |
| D53，§常见问题 | 命令找不到 / TUI 卡住用 `-g` / demo 触发工具调用方式 |

### D54：`docs/release-notes-v0.1.0.md`

> v0.1.0 发布说明 — 来源：docs

| 章节 | 内容摘要 |
| --- | --- |
| D54，§定位/§核心功能清单 | 单二进制约 **60 MiB**；无头 Agent / OpenAI 兼容 LLM / Demo / 插件系统 / 单二进制 / 内置工具 / 双模式+主题 / 非 TTY 守卫 |
| D54，§快速开始/§环境变量 | 环境变量表：`VYNTH_MODEL` gpt-4o-mini、`VYNTH_LLM_BASE_URL` https://api.openai.com/v1 |
| D54，§范围边界 In/Out | 包含：无头 agent、EchoProvider、插件（无头 `-g -p`）、单二进制、内置工具、TUI、环境变量体系；**不在**：MCP 接入、TUI 内插件加载、配置文件、预编译发行包、插件签名/市场/自动更新、联网硬隔离 |
| D54，§已知局限 | 体积约 60 MiB；demo 非真实 LLM；插件需可信；✅ 符号链接越界逃逸已修复；✅ 联网开关已修复；✅ API Key 明文端点已修复 |
| D54，§信任边界/§安全模型摘要 | 不提供进程/网络/文件系统硬隔离；信任模型=宿主权限；`run_shell` 非隔离；插件宿主完整权限；`VYNTH_API_KEY` 受端点校验保护 |
| D54，§安全审计状态 | F1–F14/A09 表：F2/F3/F5 代码已修复；F1/F4 设计信任已文档；F6/F7/F8/F11/F14/A09 为 OPEN（低优先级后续处理） |
| D54，§升级与回滚 | 单文件覆盖；保留 `dist/vynth.prev` 快照 |

### D55：`docs/rollback.md`

> 回滚 Runbook — 来源：docs

| 章节 | 内容摘要 |
| --- | --- |
| D55，§前置条件/§快照约定 | 升级前 `cp dist/vynth dist/vynth.prev`；可从 git tag 重建（仓库默认忽略 dist/） |
| D55，§回滚步骤/§回滚后验证 | 单文件覆盖；验证 `--version` 预期 0.1.0（退出码 0）+ `-g` 冒烟（退出码 0） |
| D55，§紧急快速回滚/§多版本并存/§故障排查 | 一行回滚命令；多版本重命名并存；权限/版本不符/卡住等排查表 |

### D56：`scripts/mock-llm.ts`

> 本地 mock LLM 服务 — 来源：scripts

| 章节 | 内容摘要 |
| --- | --- |
| D56，§Bun.serve | 端口默认 8787（`PORT` 可改）；`/chat/completions` 返回 SSE：首次调用发 content + `read_file` 的 `tool_calls`，二次调用发 content + stop；用于验证 `OpenAiProvider` 真实代码路径（注释标记 VYN-005） |
| D56，§运行/验证 | `bun run scripts/mock-llm.ts`；验证 `VYNTH_LLM_BASE_URL=http://localhost:8787 VYNTH_API_KEY=test ./dist/vynth -g "读一下 README"` |

---

## 3. 冲突记录

> 不同资料对同一事实描述矛盾时，**并列保留两个版本**，不做裁决（裁决权归主理人/下游）。

| 编号 | 冲突主题 | 版本 A | 出处 A | 版本 B | 出处 B | 差异说明 |
| --- | --- | --- | --- | --- | --- | --- |
| X1 | `VYNTH_LLM_BASE_URL` 默认值 | `https://api.openai.com/v1` | `D1 README §配置`；`D50 packages.md §core`（含 config.ts 默认值表）；`D51 api/overview §环境变量`；`D54 release-notes §环境变量` | `https://api.deepseek.com/v1` | `D14 config.ts §默认值`（实际生效代码）；`D53 getting-started §环境变量速查` | 文档多处写 OpenAI 默认，代码实际默认 DeepSeek；以代码为准还是以文档为准待裁决 |
| X2 | `VYNTH_MODEL` 默认值 | `gpt-4o-mini` | `D1 README §配置`；`D50 packages.md §core`；`D51 api/overview §环境变量`；`D54 release-notes §环境变量` | `deepseek-chat` | `D14 config.ts §默认值`（实际生效代码）；`D53 getting-started §环境变量速查` | 同上，模型默认值文档与代码不一致 |
| X3 | 插件 `--plugin` CLI 加载实现状态 | 「CLI 的 `--plugin 路径` 加载入口由插件加载工作流补齐」（即未接入） | `D50 packages.md §plugins 实现状态` | 「✅ 已实现」（`main.ts` 已解析 `-p/--plugin` 并经 `loadPlugin`+`activate` 接入无头模式） | `D51 api/overview §实现状态`；`D10 main.ts §parseArgs/§runHeadless`（代码确认） | packages.md 描述滞后于代码与 api/overview；以代码实现为准待确认 |
| X4 | sandbox 符号链接越界 | 「fs 路径越界守卫（可被符号链接绕过）」 | `D1 README §架构`（包结构注释） | 「已修复：`safeResolve` 现解析符号链接，cwd 内 symlink 指向沙箱外将被拒绝」 | `D34 sandbox.ts §safeResolve`（代码）；`D54 release-notes §已知局限`（✅ 符号链接越界逃逸已修复） | README 注释为陈旧描述，代码层已做 realpathSync 二次校验；二者并列保留 |
| X5 | 包管理 / workspace 声明口径 | 称「pnpm workspace + turbo」 | `D1 README §架构`；`D52 adr/0003 §决策`（「pnpm-workspace.yaml 声明 packages/* 与 apps/*」） | 根 `package.json` 同时存在 npm/bun 风格 `"workspaces": ["packages/*","apps/*"]` 字段（与 `pnpm-workspace.yaml` 并存） | `D2 package.json §workspaces`；`D3 pnpm-workspace.yaml` | pnpm 读 `pnpm-workspace.yaml`、bun/npm 读 `package.json.workspaces`；两处并存，对「实际用 pnpm 还是 bun 管理 workspace」存在口径模糊，待裁决 |
| X6 | README §配置 环境变量清单完整性 | 仅列 4 项：`VYNTH_LLM_BASE_URL`/`VYNTH_API_KEY`/`VYNTH_MODEL`/`VYNTH_MODE` | `D1 README §配置` | 应列 7 项（另含 `VYNTH_THEME`/`VYNTH_NET`/`VYNTH_DATA_DIR`）；main.ts 的 `printHelp` 也仅列 5 项（缺 `VYNTH_NET`/`VYNTH_DATA_DIR`） | `D14 config.ts`（代码实际读取全部）；`D48 docs/index.md §配置环境变量`；`D51 api/overview §环境变量`；`D53 getting-started §环境变量速查` | README 与 CLI help 文本的配置清单少于代码/其他文档；属于文档不全而非事实矛盾，仍并列保留供下游对齐 |
| X7 | 单二进制体积数值 | 约 **60 MiB** | `D54 release-notes §定位/§快速开始/§已知局限` | 约 **61MB** | `D52 adr/0003 §代价`；`D53 getting-started §编译单二进制` | 60 vs 61 数值口径差异，无实质功能冲突，供下游知悉 |

---

## 4. 硬指标清单

| 章节 | 硬指标 | 状态 |
| --- | --- | --- |
| §0 | 元信息完整（标题/版本/状态/日期/整理人/审核人/资料清单），无占位符 | ✅ |
| §1 | 每份资料有解析状态，失败/跳过注明原因（本批 56 份全部「已解析」，无失败/跳过） | ✅ |
| §2 | 每份文档按自身结构逐条摘要（D0–D56 均按文件/章节结构建表） | ✅ |
| §2 | 每条摘要标注 `D编号，§章节` 出处（见各 §2 表格首列） | ✅ |
| §3 | 冲突信息并列保留多版本，不裁决（X1–X7 均双版本并列） | ✅ |
| §4 | 硬指标清单逐项自检（本表） | ✅ |
| 全文 | 无残留占位符/示例前缀/待补充标记（定稿纪律已满足，详见附录 A） | ✅ |
| 结构 | 核心章节 §0~§4 + 附录 A/附录 B 齐全，未删减 | ✅ |
| 溯源 | 事实可追溯到「文件 + 章节/位置」（每条含 D编号,§章节） | ✅ |

---

## 附录 A：生成流程

### 流程总览

| 步骤 | 动作 | 落入章节 |
| --- | --- | --- |
| Step0 | 读取模板 `material_digest.md` + 主理人转交的全部原始资料（D0–D56） | — |
| Step1 | 盘点资料清单，标注类型/来源/解析状态 | §1 |
| Step2 | 逐份打开资料，按自身章节/结构逐条摘要，标注 `D编号，§章节` | §2 |
| Step3 | 交叉比对不同资料，发现并记录矛盾（X1–X7） | §3 |
| Step4 | 逐项核验硬指标 | §4 |

```mermaid
flowchart LR
    S0[读取模板与资料] --> S1[盘点资料清单]
    S1 --> S2[逐份精读逐章节摘要]
    S2 --> S3[交叉比对记录冲突]
    S3 --> S4[硬指标自检]
```

### 整理原则

1. **逐份精读，不跨文档归并**：摘要按文档自身结构组织，不做跨文档的主题重组（那是下游的事）。
2. **出处即章节号**：每条摘要标注 `D编号，§章节`，直接映射回原文位置。
3. **冲突保留**：矛盾信息并列保留两个版本，不擅自裁决。
4. **事实驱动**：以原始资料中的事实为准，不添加主观推断；推断类内容已在 §2 中标注「综合归纳/推断」字样并在 §3 单独记录矛盾。
5. **不越权**：本阶段仅做资料摄入与结构化，未对业务边界/技术选型/架构决策下结论。

---

## 附录 B：解析 Skill

> 本批次资料全部为文本型（源码 `.ts`、配置 `.json`/`.yaml`、文档 `.md`），统一以直读方式解析，**未调用任何二进制 Office/PDF Skill**。下表按本项目的实际资料类型改写，表头保留。

| 资料类型 | 本项目对应资料 | 解析方式 |
| --- | --- | --- |
| `md`（文档） | D1 README、D48–D55 docs/* | 直读 markdown，按标题结构摘要 |
| `ts`（源码） | D10/D13–D14/D16–D18/D21–D24/D27–D30/D33–D34/D37–D38/D41–D43/D46–D47/D56 | 直读 TypeScript 源码，按文件/导出符号摘要 |
| `json`（配置） | D2/D4–D6/D8/D11–D12/D19–D20/D25–D26/D31–D32/D35–D36/D39–D40/D44–D45 | 直读 JSON，按 key 结构摘要 |
| `yaml`（配置） | D3 pnpm-workspace.yaml | 直读 YAML，按字段摘要 |
| `text`（忽略规则） | D7 .gitignore | 直读文本 |
| （未使用）`docx` | — | 本次无 Word 资料 |
| （未使用）`pdf` | — | 本次无 PDF 资料 |
| （未使用）`pptx` | — | 本次无 PPT 资料 |
| （未使用）`xlsx` | — | 本次无 Excel 资料 |
