# Package 职责详解

> 替代原 `crates.md`。每个 `@vynth/*` 包的职责、关键文件与对外契约。路径相对于仓库根。

---

## `@vynth/core` — 共享基础（`packages/core`）

所有包的公共地基，无业务依赖（叶子包）。

| 文件 | 职责 |
|------|------|
| `src/types.ts` | 全局类型：`Mode`（`plan`/`vibe`）、`ChatMessage`、`ToolParam`/`ToolDef`/`ToolResult`/`ToolCall`、`StreamEvent`（`token`/`tool`/`done`）、`VynthConfig` |
| `src/config.ts` | `loadConfig(overrides?)`：从 `process.env` 读取并合并默认值（**不读 config 文件**） |
| `src/errors.ts` | 错误体系：`VynthError` 基类 + `ConfigError`/`LlmError`/`ToolError`/`SandboxError`/`McpError`/`PluginError`（均带 `code`） |
| `src/events.ts` | `Emitter<M>` 类型安全事件总线：`on(key, fn)` 返回退订函数，`emit(key, payload)` |
| `src/logger.ts` | `log(level, msg, meta?)` 分级日志（debug/info/warn/error），`setLogLevel` |
| `src/index.ts` | 统一再导出 |

**配置默认值（`loadConfig`）**：`mode=vibe`、`llmBaseUrl=https://api.deepseek.com/v1`、`apiKey=`(空)、`model=deepseek-v4-pro`、`theme=mocha`、`sandbox.networkAllowed = VYNTH_NET !== '0'`、`sandbox.cwd = process.cwd()`、`dataDir = ~/.vynth`。

---

## `@vynth/engine` — LLM + 工具 + Agent（`packages/engine`）

合并原 `llm` + `tools` + `agent` 三块；依赖 `core`、`sandbox`。

| 文件 | 职责 |
|------|------|
| `src/llm.ts` | `LLMProvider` 接口 + `createProvider(config)`；`OpenAiProvider`（OpenAI 兼容 SSE 客户端，逐行解析 `data:` 帧，累加 `tool_calls`）；空 `apiKey` 时抛出 `LlmError` |
| `src/tools.ts` | `ToolRegistry`（register/get/list/run，重名抛 `ToolError`，未知工具返回 `ok:false`）；`builtinTools(cwd)` 注册 `read_file`/`write_file`/`run_shell` |
| `src/agent-loop.ts` | `runAgent(goal, opts)`：异步生成器，按 `maxSteps`（默认 8）循环 `provider.chat` → 收集 token/tool → `tools.run` → 回填 `messages` |
| `src/index.ts` | 统一再导出 |

**对外契约**：
- `LLMProvider.chat(messages, tools): AsyncIterable<StreamEvent>`
- `runAgent(goal, { provider, tools, system?, maxSteps? }): AsyncGenerator<StreamEvent>`
- 工具定义遵循 `core` 的 `ToolDef`（`run` 可同步或异步返回 `ToolResult`）。


---

## `@vynth/tui` — 轻量 ANSI TUI（`packages/tui`）

**非 ink**：自研 ANSI 渲染器，规避 `yoga.wasm` 无法被 `bun build --compile` 打包的问题。依赖 `core`、`engine`、`ansi-escapes`。

| 文件 | 职责 |
|------|------|
| `src/tui.ts` | `startTui(config)`：raw mode + `readline` + keypress 事件；全屏重绘（`\x1b[2J\x1b[H`）；`runAgent` 流式 → `StreamArea` 直写；`ctrl-c`/`ctrl-d` 退出（恢复 raw mode） |
| `src/stream-escape-hatch.ts` | `StreamArea`：流式逃生舱，用 `ansi-escapes` 行光标回退 + 清行直写，避免每个 token 触发全屏重绘 |
| `src/theme.ts` | `palette(theme)` 返回 Catppuccin `mocha`/`latte` 调色板；`fg`/`bg` 真彩 ANSI 转义；`reset` |
| `src/index.ts` | 导出 `startTui`、`StreamArea`、`palette`/`fg`/`bg`/`reset` |

**行为要点**：仅当 `process.stdin.isTTY` 时进入 raw mode；否则退化为普通输入。`draw()` 每帧清屏重绘历史 + 实时流 + 输入行。

---

## `@vynth/sandbox` — 隔离执行（`packages/sandbox`）

所有工具对宿主 fs / shell 的访问出口。依赖 `core`。

| 文件 | 职责 |
|------|------|
| `src/sandbox.ts` | `safeResolve(target, cwd)`：路径越界守卫（解析后必须落在 `cwd` 内，否则抛 `SandboxError`）；`readText`/`writeText`；`runCommand(command, { cwd, networkAllowed?, timeoutMs? })`：spawn `sh -c`，网络被禁时直接拒绝，默认超时 30s（`SIGKILL`） |
| `src/index.ts` | 统一再导出 |

**不变量**：工具不得绕过 `sandbox` 直接 `fs`/`child_process`；网络默认开启（`VYNTH_NET !== '0'` 时关闭）。

---

## `@vynth/mcp` — MCP 客户端（`packages/mcp`）

stdio JSON-RPC 的 Model Context Protocol 客户端。依赖 `core`。

| 文件 | 职责 |
|------|------|
| `src/mcp-client.ts` | `McpClient`：spawn 子进程（stdio），`connect()` 发 `initialize`（protocolVersion `2024-11-05`）→ `tools/list`；`callTool(name, args)` 发 `tools/call`；按行解析 JSON-RPC 响应，按 `id` 匹配 pending |
| `src/index.ts` | 导出 `McpClient` |

**实现状态**：客户端已可用，但尚未在 `apps/cli` 的 agent 运行路径中接入（接入后 MCP 工具可并入 `ToolRegistry`）。

---

## `@vynth/plugins` — 插件体系（`packages/plugins`）

动态加载第三方 / 本地工具扩展。依赖 `core`、`engine`。

| 文件 | 职责 |
|------|------|
| `src/loader.ts` | `Plugin` 接口（`name` + `activate(reg: ToolRegistry)`）；`loadPlugin(entryPath)`：`import(pathToFileURL(...))` 动态加载，要求导出 `pluginName` 与 `activate`；`loadAll(entries, reg)` 批量激活 |
| `src/index.ts` | 导出 `loadPlugin`/`loadAll`/`Plugin` |

**契约**：插件模块默认导出 `pluginName: string` 与 `activate(reg: ToolRegistry): void`；`activate` 内向 `reg` 注册工具即可被 agent 调用。

**实现状态**：`loader` 已就绪；CLI 的 `--plugin <path>` 加载入口由插件加载工作流补齐（见 [API 总览](/api/overview.md#实现状态)）。

---

## `@vynth/harness` — 集成测试（`packages/harness`）

e2e / 集成测试驱动，私包（`private: true`）。依赖 `core`、`engine`。

| 文件 | 职责 |
|------|------|
| `src/e2e.test.ts` | `bun:test` 用例：用 `MockProvider` 驱动 `runAgent`，断言「流式 token + 工具调用 + 工具被执行」全链路 |

**运行**：`bun test`（或 `turbo test`，`apps/cli` 的 `test` 脚本为 `bun test packages`）。

---

## `@vynth/cli` — CLI 应用（`apps/cli`）

唯一带 `bin` 的包，聚合 `core` + `engine` + `tui`。

| 文件 | 职责 |
|------|------|
| `src/main.ts` | `parseArgs`（`-g/--goal`、`-m/--mode`、`-v/--version`、`-h/--help`）；`printHelp`；`runHeadless(goal)`（无头流式）；`main()` 分发到 TUI 或无头 |
| `package.json` | `bin: { vynth: src/main.ts }`；`build` = `bun build --compile ... --outfile ../../dist/vynth` |

**分发**：`bun run compile` 产出单二进制 `dist/vynth`；`--version` 输出 `0.1.0`。
