# Vynth v0.1.0 发布说明

> 首次发布 · 版本 `0.1.0` · 单二进制 `dist/vynth`（约 60 MiB）

## 一句话定位

**Vynth 是你 terminal 里的代码合成器**——纯 TypeScript 编写、以单个 Bun 二进制（`dist/vynth`，约 60 MiB）零依赖分发，把自然语言目标「合成」成代码。面向在 shell 工作流里想把一句话变成可运行代码的开发者：无需离开终端、无需搭脚手架，无头（`-g`）或交互（TUI）两种形态直接使用。

## 核心功能清单

- **无头 Agent 模式**：`./dist/vynth -g "<目标>"` 触发 agent 循环，结果流式直写 stdout；无需 TTY，可在管道 / CI 中使用。
- **OpenAI 兼容 LLM**：`OpenAiProvider` 走 SSE 流式，逐 token 解析 + `tool_calls` 工具调用；兼容任意 OpenAI 兼容端点（含本地 `/v1`、第三方服务）。
- **Demo 模式**：未设置 `VYNTH_API_KEY` 时自动启用 `EchoProvider`，离线体验流式输出与工具循环，开箱即跑。
- **插件系统**：`vynth -g "<目标>" -p <path>` 动态加载本地 `.ts/.js` 插件，经 `activate(reg)` 注册自定义工具，扩展 agent 能力。
- **单二进制分发**：`bun build --compile` 产出单一可执行文件，无 node_modules、无外部 wasm、无运行时依赖（仅需 Bun 构建环境）。
- **内置工具**：`read_file` / `write_file`（路径受 cwd 越界守卫约束）、`run_shell`（`sh -c` 以**宿主权限**运行，仅设置工作目录、无命令 / 文件系统隔离）。
- **双模式 + 主题**：`plan`（先规划）/`vibe`（边聊边写），Catppuccin `mocha` / `latte` 主题。
- **非 TTY 守卫**：非交互终端下运行 TUI 时退出码 `2` 并提示改用无头模式。

## 快速开始

### 1. 环境要求与构建

```bash
# 需要 Bun >= 1.1
bun install
bun run compile          # → 产出 dist/vynth（当前约 60 MiB）
```

### 2. 无需 Key 先跑 Demo

```bash
./dist/vynth -g '用一句话介绍 vynth'
```

### 3. 接入真实 LLM

```bash
export VYNTH_API_KEY="sk-..."                            # 必填：真实 LLM key
export VYNTH_MODEL="gpt-4o-mini"                         # 可选
export VYNTH_LLM_BASE_URL="https://api.openai.com/v1"    # 可选：兼容端点
./dist/vynth -g '给当前目录写一份 README.md'
```

### 4. 加载插件

```bash
./dist/vynth -g '用 hello 工具向世界问好' -p packages/plugins/examples/hello-plugin.ts
```

插件入口需 `export const pluginName` 与 `export function activate(reg)`。详见插件示例 `packages/plugins/examples/hello-plugin.ts`。

### 环境变量

| 变量 | 作用 | 默认 |
|------|------|------|
| `VYNTH_API_KEY` | LLM key（空 = demo） | 空 |
| `VYNTH_MODEL` | 模型名 | `gpt-4o-mini` |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.openai.com/v1` |
| `VYNTH_MODE` | `plan` \| `vibe` | `vibe` |
| `VYNTH_THEME` | `mocha` \| `latte` | `mocha` |
| `VYNTH_NET` | 沙箱网络开关（`'0'` = 禁网，尽力而为） | 开启 |
| `VYNTH_DATA_DIR` | 数据目录 | `~/.vynth` |

## v0.1.0 范围边界

### 本次包含（In）

- 无头 agent（`-g`）+ OpenAI 兼容 SSE 流式与工具调用
- `EchoProvider` demo 模式
- 插件加载（**无头模式** `-g` + `-p`）
- 单二进制分发（`bun build --compile`）
- 内置工具（`read_file` / `write_file` 路径越界守卫；`run_shell` 以宿主权限运行、无隔离）
- 交互式 TUI（需真实 TTY）、双模式、Catppuccin 主题、非 TTY 守卫
- 环境变量配置体系

### 明确不在范围内（Out，本版不做）

- **MCP 接入**：`@vynth/mcp` 的 `McpClient` 已就绪，但**尚未接入 CLI** agent 工具集，本版不暴露 MCP 工具。
- **TUI 内插件加载**：插件加载目前仅支持**无头模式**；交互式 TUI 暂不支持运行时加载插件。
- **配置文件**：配置仅经环境变量注入，**不读取任何 `config.toml` / 配置文件**。
- **预编译发行包**：本版需从源码 `bun build --compile` 自编译，不提供多平台预编译二进制下载。
- **插件签名 / 市场 / 自动更新**：无插件签名验证、无插件市场、无自动更新机制。
- **联网硬隔离**：本版不提供进程级硬网络隔离；`run_shell` 联网现受 `VYNTH_NET='0'` 网关阻断，但仍为软网关、非硬安全边界（见信任边界）。

## 已知局限

- **二进制体积较大**：当前约 60 MiB（含未优化的运行时体积），目标收敛至 20–40 MiB。
- **Demo 非真实 LLM**：`EchoProvider` 仅用于体验流式与工具循环，不调用真实模型，需自备 API key 才有实际能力。
- **插件需可信来源**：插件经动态 `import()` 执行任意代码（详见下方信任边界），仅可从你信任的来源加载。
- ✅ **符号链接越界逃逸 — 已在 v0.1.0 修复**：`safeResolve` 现解析符号链接，cwd 内 symlink 指向沙箱外将被拒绝。
- ✅ **联网开关 — 已在 v0.1.0 修复**：`run_shell` 现受 `VYNTH_NET='0'` 阻断，不再对 shell 失效。
- ✅ **API Key 明文端点 — 已在 v0.1.0 修复**：拒绝向远程明文 `http` 端点发送 API Key（localhost 放行以保留本地调试）；非默认端点打印告警。
- **构建依赖 Bun**：需 Bun >= 1.1 环境构建；运行单二进制本身无需额外依赖。

## ⚠ 信任边界（务必阅读）

> **Vynth 的插件系统会执行任意代码。**

当使用 `-p <path>` 加载插件时，Vynth 通过动态 `import()` 直接加载并执行文件中的代码（`activate(reg)` 在加载时即运行），**插件代码与 Vynth 本体运行在同一运行时、拥有同等权限**：

- 可读取 / 修改你文件系统上的**任意**文件（不受内置工具 cwd 沙箱约束）；
- 可读取**所有环境变量**，包括 `VYNTH_API_KEY` 等敏感凭据；
- 可发起**任意网络请求**、执行**任意命令**。

**Vynth 不对插件代码做沙箱隔离或权限限制。** 因此：

- ✅ **仅从你完全信任的来源加载插件**（自己的代码、团队审计过的代码）。
- ❌ **不要加载来源不明、未经审查的插件**——恶意插件可窃取凭据、破坏文件、远程控制。
- 🔒 在共享 / CI 环境中使用 `-p` 前，确认插件路径与内容可信。

内置 `read_file` / `write_file` 受 cwd 越界守卫约束；`run_shell` 以**宿主权限**运行（`sh -c`，仅设置工作目录、无命令 / 文件系统隔离）；**插件注册的自定义工具同样不受沙箱约束**。工具的能力边界 = 你的进程能力边界。

另：联网开关（`VYNTH_NET='0'`）现已生效，`run_shell` 受其为阻断；但仍属软网关、**非进程级硬安全边界**，勿将其作为不可信环境的唯一隔离保证。

## 安全模型摘要

本版 Vynth **不提供进程 / 网络 / 文件系统的硬隔离**，其安全模型建立在「用户以自身宿主权限运行」这一前提上：

- **信任模型 = 宿主权限**：agent 循环、内置工具与插件均以你的用户权限运行；Vynth 不隔离、也不降低这些权限。
- **`run_shell` 非隔离**：直接以 `sh -c` 在宿主执行命令，仅设置工作目录，无命令 / 文件系统沙箱。
- **插件以宿主完整权限运行**：`-p` 经动态 `import()` 加载的代码可读取任意文件、读取所有环境变量（含 `VYNTH_API_KEY`）、发起任意网络请求、执行任意命令——仅加载可信来源。
- **`VYNTH_API_KEY` 发往所配置的 LLM 端点**：请求地址由 `VYNTH_LLM_BASE_URL` 决定，并受端点校验保护——**已拒绝向远程明文 `http` 端点发送密钥**（localhost 放行以保留本地调试），非默认端点会打印告警。仍建议仅配置你信任的 `https` 端点。

## 安全审计状态

以下为安全审计（OWASP/STRIDE）发现的逐项状态，便于评审一眼区分「已文档披露」与「代码层仍 OPEN」。完整审计报告见 security-officer。

| 发现 | 代码 / 修复状态 | 文档状态 |
|------|----------------|----------|
| **F1** — 插件 `import()` 无签名、宿主完整权限 | OPEN（设计信任模型，已文档化） | 已披露 |
| **F2** — sandbox / `run_shell` 无隔离 + `VYNTH_NET=0` 对 shell 失效 | **已修复**（networkAllowed 透传已接线于 main.ts:51 与 tui.ts:14；无隔离为设计信任模型，已文档化） | 已披露 |
| **F3** — `read_file` / `write_file` 符号链接逃逸（`safeResolve` 未 `realpath`） | **已修复**（safeResolve 加 realpathSync 二次校验） | 已覆盖 |
| **F4** — 不可信 LLM 端点 `tool_call` 注入 → RCE | OPEN（设计层面，已文档化信任模型） | 部分披露 |
| **F5** — `VYNTH_API_KEY` 无校验外发（http 明文 / 钓鱼） | **已修复**（assertSafeEndpoint: URL 校验 + 拒 http 明文 + 非默认端点告警） | 已披露 |
| **F6** — `.env` 未纳入 `.gitignore` | OPEN | 未涉及 |
| **F7** — LLM `fetch` 无超时 / SSE 缓冲无上限 | OPEN | 未涉及 |
| **F8** — 工具参数缺类型 / 必填校验 | OPEN | 未涉及 |
| **F11** — 无头模式打印工具参数可能泄露 | OPEN | 未涉及 |
| **F14** — `loadAll` 单插件异常致整体失败 | OPEN | 未涉及 |
| **A09** — 安全事件无审计日志 | OPEN | 未涉及 |

> 本版 **F2 / F3 / F5 已在代码层修复**；F1 / F4 为设计信任模型（已文档化）；F6 / F7 / F8 / F11 / F14 / A09 仍为低优先级 OPEN，计划在后续版本处理。F9 / F10 / F12 / F13 非独立发现，不在表中。

## 升级与回滚

Vynth 为单二进制分发，升级即覆盖：

```bash
# 升级前保留上一版本快照（推荐）
cp dist/vynth dist/vynth.prev

# 重新构建并覆盖
bun run compile        # 覆盖 dist/vynth

# 若需回滚，将快照覆盖回去即可
cp dist/vynth.prev dist/vynth
```

- 二进制为单文件，无数据库 / 配置迁移；回滚只需恢复可执行文件本身。
- 数据目录 `~/.vynth`（`VYNTH_DATA_DIR` 可改）跨版本兼容，回滚一般不会影响。
- 多版本并存：可将不同版本重命名为 `vynth-0.1.0` / `vynth-0.2.0` 并分别放入 `PATH`。
