# Vynth v0.2.0 发布说明

> 本次发布 · 版本 `0.2.0` · 单二进制 `dist/vynth`（实测 60.51 MiB）
> 完整变更日志见根目录 `CHANGELOG.md`。

## 一句话定位

**Vynth 是你 terminal 里的代码合成器**——纯 TypeScript / Bun 编写、以单个
Bun 单二进制（`dist/vynth`，实测 **60.51 MiB**）零依赖分发，把自然语言目标
「合成」成代码。面向在 shell 工作流里想把一句话变成可运行代码的开发者：
无需离开终端、无需搭脚手架，无头（`-g`）或交互（TUI）两种形态直接使用。

## 本次新增（Sprint 1–6）

### ✅ Agent 引擎 + LLM（F4 / F6 / F7 / F8）

- `OpenAiProvider` 走 OpenAI 兼容 SSE，逐 token 解析 `delta.content` +
  `delta.tool_calls`，并解析顶层 `usage` 计费字段；非默认端点告警。
- `runAgent` 实现 agent 循环（`maxSteps`，默认 8），流式把 token / tool /
  done 事件交给调用方。
- **Demo 模式**：`VYNTH_API_KEY=""` 时自动切换 `EchoProvider`，离线体验
  流式输出 + 工具调用回填，开箱即跑。
- 默认 LLM：`https://api.deepseek.com/v1` + `deepseek-chat`（与冻结裁决 X1/X2 一致）。
- 端到端验证：`scripts/mock-llm.ts` 起本地 OpenAI 兼容 SSE 服务后，
  `VYNTH_LLM_BASE_URL=http://localhost:8787 VYNTH_API_KEY=test ./dist/vynth -g ...`
  可在无真实 Key 下走真实 SSE 客户端代码路径。

### ✅ 内置工具 + 沙箱守卫（F5 / F10）

- `read_file` / `write_file` 经 `safeResolve` 拒绝 `../` 越界、cwd 外绝对
  路径、以及 realpath 后的 symlink 逃逸（**X3 攻击面 100% 拦截**）。
- `run_shell`（宿主权限执行命令）受 `VYNTH_NET='0'` 阻断联网；允许联网时
  正常执行。
- `@vynth/sandbox` 7 例单测覆盖：cwd 内可读、`../` 拒绝、绝对路径拒绝、
  symlink 拒绝、writeText 越界拒绝、VYNTH_NET off / on 两种行为。

### ✅ 插件无头接入（F9）

- `loadPlugin(entryPath)` 动态 `import()` 加载并校验 `pluginName` / `activate`；
  `loadAll(entries)` 批量 activate 并返回已加载名。
- 三类异常路径（缺 `activate`、缺 `pluginName`、无法 import）均抛
  `PluginError`。
- 6 例包级单测 + 4 个 fixture（good / good-2 / bad-no-activate /
  bad-no-name）。

### ✅ TUI 双模式契约（F2 / F3）

- `startTui(config)`：交互 TTY 模式；`readline` + raw mode + ANSI 重绘；
  Catppuccin `mocha` / `latte` 主题；Ctrl+C / Ctrl+D 退出。
- `runHeadless(goal, plugin?)`：无头模式；流式 token + 工具调用 stdout
  直写；不依赖 TTY，可在管道 / CI 使用。
- **非 TTY 守卫**：非交互终端下未传 `-g` 时退出码 `2` 提示改用无头模式；
  headless 模式不依赖 TTY。
- `@vynth/tui` 10 例包级单测（theme 5 + StreamArea 5）；harness
  e2e 增加 2 例 CLI TUI 分流契约。

### ✅ 工程规矩闸门

- `.github/workflows/ci.yml`：CI 8 阶段（install → lint → build → compile
  → test → gitleaks → 体积门禁 → sign/publish）。
- `gitleaks.toml`：密钥硬编码红线，CI 默认 fail。
- `scripts/check-binary-size.ts`：单二进制体积门禁（MVP ≤ 61 MB）。
- `docs/dev-guide.md`：分支模型、Conventional Commits、错误码规范、安全
  红线、冻结值表、错误码 6 位码迁移立项。

### ✅ 可观测 / 度量

- 冷启动 `scripts/bench-cold-start.ts`：测首字节延迟分布，
  **实测 P95 = 30.5 ms ≤ 150 ms 基线**（10 次采样）。
- 体积 / 启动指标纳入 CI 历史基线（手动跑 `bun ...` 验证）。

## 已知 F1 / F4 设计信任模型（仍 OPEN，文档化）

> 完整审计 + 历史修复状态见 `docs/release-notes-v0.1.0.md` 与
> `delivery/安全设计.md`。

| 项 | 状态 |
|---|---|
| **F1** — 插件 `import()` 无签名、宿主完整权限 | OPEN（设计信任模型，仅加载可信来源） |
| **F4** — 不可信 LLM 端点 `tool_call` 注入 → RCE | OPEN（设计层面，需审计 LLM 来源） |
| **F6** — `.env` 未纳入 `.gitignore` | 已修（`.gitignore` 含 `.env*` 与 `.vynth/`） |
| **F7** — LLM `fetch` 无超时 / SSE 缓冲无上限 | OPEN（下个补丁修复） |
| **F8** — 工具参数缺类型校验 | 部分修复（`@vynth/core/types.ts` 已声明 required，下版升 typecheck） |
| **F11** — 无头打印工具参数可能泄露 | OPEN（未来加 redact 规则） |
| **F14** — `loadAll` 单插件异常致整体失败 | OPEN（设计权衡，本次未改） |
| **A09** — 安全事件无审计日志 | OPEN（与 F7 同批） |
| **错误码 6 位码迁移** | OPEN（dev-guide 已立项，strings 域暂保留） |

> 后续在 v0.2.x patch 发布中逐项收敛。

## 快速开始

```bash
bun install
bun run compile                                # → dist/vynth (≈ 60 MiB)

# Demo（无 API key）
./dist/vynth -g '一句话介绍 vynth'

# 接入真实 LLM
export VYNTH_API_KEY="sk-..."
export VYNTH_MODEL="deepseek-chat"             # 默认，可改
export VYNTH_LLM_BASE_URL="https://api.deepseek.com/v1"
./dist/vynth -g '把 README 重写得更紧凑'

# 加载插件（仅无头模式）
./dist/vynth -g '用 hello 工具向世界问好' -p packages/plugins/examples/hello-plugin.ts

# 交互 TUI（需真实 TTY）
./dist/vynth
```

## 环境变量

| 变量 | 作用 | 默认 |
|------|------|------|
| `VYNTH_API_KEY` | LLM key（空 = demo） | 空 |
| `VYNTH_MODEL` | 模型名 | **`deepseek-chat`**（v0.1.0 历史值为 `gpt-4o-mini`，v0.2.0 修正为冻结值） |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点 | **`https://api.deepseek.com/v1`**（OpenAI 兼容） |
| `VYNTH_MODE` | `plan` \| `vibe` | `vibe` |
| `VYNTH_THEME` | `mocha` \| `latte` | `mocha` |
| `VYNTH_NET` | `run_shell` 联网开关（`'0'` = 禁网） | 开启 |
| `VYNTH_DATA_DIR` | 数据目录 | `~/.vynth` |

## 升级与回滚

Vynth 为单二进制分发，升级即覆盖：

```bash
# 升级前保留上一版本快照
cp dist/vynth dist/vynth.prev

# 重新构建并覆盖
bun run compile

# 回滚（需要快照）
cp dist/vynth.prev dist/vynth
```

- 二进制为单文件，无数据库 / 配置迁移；数据目录 `~/.vynth`（`VYNTH_DATA_DIR`
  可改）跨版本兼容。
- 多版本并存：可将不同版本重命名为 `vynth-0.1.0` / `vynth-0.2.0` 并分别
  放入 `PATH`。

## CI / 闸门

`main` 分支每次 push 触发 GitHub Actions 8 阶段：

```
install → lint → build → compile → test → gitleaks → 体积门禁 → sign/publish
```

- **lint**：`biome check .`，0 error 才放行。
- **compile**：`bun run compile` 产 `dist/vynth`。
- **test**：`bun test packages`，58 例全绿后放行。
- **gitleaks**：密钥硬编码扫描（参考 `gitleaks.toml`）。
- **体积门禁**：`bun scripts/check-binary-size.ts`，> 61 MB 拒绝合并。
- **sign/publish**：仅当 `github.ref` 以 `refs/tags/v*` 开头时触发（codesign +
  notarize / gpg sign，本版本预留入口）。

## 仓库

- GitHub: `https://github.com/Agions/vynth`（从 `synerix` 改名为 `vynth`，旧链接 301 跳转）
- 发布：每次 push `main` 跑 CI；正式发布以 `vX.Y.Z` tag 触发 sign/publish。
