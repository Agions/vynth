# Changelog

All notable changes to Vynth are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-07-25 — MVP 完整闭环上线

> GitHub 仓库上线：`Agions/vynth`（从 `synerix` 改名为 `vynth`）。

### Added
- **Agent 引擎 + LLM（F4 / F6 / F7 / F8）**：`@vynth/engine` 提供 `runAgent` 单步上限循环
  （`maxSteps`，默认 8）、`OpenAiProvider` 解析 OpenAI 兼容 SSE（含 `tool_calls` 与顶层
  `usage`）、`EchoProvider` 在无 `VYNTH_API_KEY` 时自动启用（demo 模式）；新
  `scripts/mock-llm.ts` 可在本地起真实 OpenAI 兼容 SSE 服务端用于联调。
- **内置工具 + 沙箱守卫（F5 / F10）**：`@vynth/sandbox` 的 `safeResolve`
  拒绝 `../` 越界、cwd 外绝对路径，以及对 realpath 后的 symlink 逃逸；
  `run_shell` 受 `VYNTH_NET='0'` 阻断联网；`runCommand`/`readText`/`writeText`
  全部经统一守卫。
- **插件无头接入（F9）**：`@vynth/plugins` 的 `loadPlugin` 动态 `import()`
  加载并校验 `pluginName` / `activate`；`loadAll` 批量激活并返回已加载名；
  6 例包级单测覆盖合法加载、批量加载、缺导出与无法 import 四类异常路径。
- **TUI 双模式契约（F2 / F3）**：`@vynth/tui` 的 `startTui` 与 CLI `runHeadless`
  分别承担交互 TUI 与无头模式；非 TTY 守卫——非交互终端无 `-g` 时退出码 `2`
  并提示改用无头模式；headless 不依赖 TTY，可在管道 / CI 使用。
- **工程规矩闸门（CI 8 阶段）**：`.github/workflows/ci.yml` 涵盖 install → lint
  → build → compile → test → gitleaks → 体积门禁 → sign/publish（仅 tag）；
  `gitleaks.toml` 锁定密钥硬编码红线；`scripts/check-binary-size.ts` 单二进制
  体积门禁（MVP ≤ 61 MB）；`docs/dev-guide.md` 统一分支模型、Conventional
  Commits、错误码规范、安全红线、冻结值表。
- **冷启动基线测量**：`scripts/bench-cold-start.ts` 输出 P50 / P95 / max，
  实测 **P95 = 30.5 ms ≤ 150 ms 基线**（10 次采样）。
- **CLI 退出码契约（F11）**：未知参数 / `-g` 缺值 / `-m` 非法值统一退出码 `2`
  并 stderr 提示；`--version` / `--help` 退出码 `0`；空 goal 在非 TTY 退出码 `2`。
- **架构交付归档**：`delivery/` 收纳 G1–G6 七道阶段门的全部产物
  （material_digest / research_report / 高层架构 / 系统设计 / UserStory /
  部署设计 / 安全设计 / G6_交付汇总）。
- **实施开发计划**：`docs/实施开发计划.md` 串联 G0 启动 → G6 交付 → Sprint
  1–6 实施，形成可执行路线图。

### Changed
- `apps/cli` 与所有 `packages/*` 同步升级至 **`0.2.0`**。
- 默认模型由 `deepseek-v4-pro` 修正为 **`deepseek-chat`**（与 X1/X2 冻结裁决一致）；
  默认端点 `https://api.deepseek.com/v1`（OpenAI 兼容）。
- `docs/guide/getting-started.md` 体积与冷启动描述由历史值改为实测值
  （**60.51 MB** / **P95 30.5 ms**）。
- CLI `--help` 默认模型描述由历史 `gpt-4o-mini` 改为 `deepseek-chat`。

### Security
- **Sprint 1**: CLI 用法错误统一退出码 `2`（之前静默忽略，现显式拒绝未知
  参数 / 缺值 / 非法值）。
- **Sprint 2**: `OpenAiProvider` 拒绝向远程明文 `http` 端点发送 API Key
  （localhost 放行）；非默认端点告警。
- **Sprint 3**: 沙箱守卫覆盖 `..` 越界、绝对路径越界、symlink 逃逸三类逃逸
  路径（**F10 对抗 X3** 单测 100% 拦截），并经 `VYNTH_NET='0'` 阻断 `run_shell`
  联网。
- **Sprint 6**: gitleaks 密钥扫描入 CI 红线；体积门禁防回归。

### Test
- `bun test packages`: **58 pass / 0 fail**
  - `@vynth/core`: 15 例（loadConfig 默认值、VYNTH_NET 解析、错误码层级、
    Emitter、log 分级）
  - `apps/cli`: 5 例（退出码契约）
  - `@vynth/harness`: 10 例（agent 流式工具、SSE 解析、plugin、EchoProvider、
    sandbox 守卫、symlink、联网拒止、明文 http 拒绝、CLI 退出码、CLI TUI 分流）
  - `@vynth/engine`: 5 例（无 Key EchoProvider、runAgent 流式与终止、
    maxSteps、OpenAI SSE、http 拒绝）
  - `@vynth/sandbox`: 7 例（cwd 内可读、../、绝对路径、symlink、writeText 越界、
    VYNTH_NET 开/关）
  - `@vynth/plugins`: 6 例（合法加载、批量加载、三类异常路径）
  - `@vynth/tui/theme`: 5 例（mocha/latte 色值、fg/bg/reset）
  - `@vynth/tui/stream-escape-hatch`: 5 例（首次不擦线、二次擦、clear 归零、
    空 clear、空串）
- `bun run lint`: 0 error（57 文件）
- `bun run compile` + 体积门禁: **60.51 MB < 61 MB PASS**
- `BENCH_RUNS=10 BENCH_LIMIT_MS=150 bun scripts/bench-cold-start.ts`:
  P95 = 30.5 ms ≤ 150 ms PASS

## [0.1.0] - 2026-07-24

### Added
- 首次发布——本地优先单二进制 TUI 编程工具的骨架：`@vynth/core` /
  `engine` / `sandbox` / `tui` / `plugins` / `mcp` / `harness` + `apps/cli`。
- 无头 Agent 模式 (`vynth -g "<目标>"`)、OpenAI 兼容 LLM、
  EchoProvider demo、`read_file` / `write_file` / `run_shell` 内置工具、
  交互 TUI（Catppuccin mocha/latte 主题）、Catppuccin 主题、
  环境变量配置体系。
- 初始发布说明 `docs/release-notes-v0.1.0.md`。

[Unreleased]: https://github.com/Agions/vynth/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Agions/vynth/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Agions/vynth/releases/tag/v0.1.0
