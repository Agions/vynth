# Changelog

All notable changes to Vynth are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-07-27 — TypeScript strict 合规 + TUI 全帧重绘 + 3 段式 IDE 布局

### Added
- **TypeScript strict mode 全量合规**：8 个 workspace 全部通过 `tsc --noEmit`；
  新增 `@types/node` + `@types/bun` 依赖；修复 `packages/core` / `packages/mcp` /
  `packages/sandbox` 的 strict 错误（null guard、Pick 类型、default code 映射）。
- **TUI 全帧重绘 + 3 段式 IDE 布局**：废弃 DECSTBM 增量绘制 + SGR 颜色泄漏，
  改用 `ESC[2J` 全帧重绘消除乱码；顶部固定状态栏、中部可滚动聊天区、底部
  4 行矩形输入框；移除角色标签（System/Vynth/You/Tool），改用左侧颜色条；
  新增 CJK 感知宽度计算、2000 行 scrollback 环形缓冲区、SGR 1006 鼠标滚轮支持；
  新增 `render.ts`（`renderBadge` / `renderStatusBar` / `renderPanel` /
  `renderMessage` / `renderToolBlock` / `renderInputBox` / `clipHistory`）。

### Changed
- `packages/tui/src/tui.ts`：完整重写为 3 段式布局 + 全帧重绘循环；
  鼠标追踪启用 SGR 1006 协议；滚轮事件映射 scrollback 偏移。
- `packages/tui/src/theme.ts`：补充 TUI 全帧渲染所需色值常量。
- `packages/tui/src/stream-escape-hatch.ts`：适配全帧重绘逃生舱路径。
- CI 阶段 2 增加 8 个 workspace 的 `bun x tsc --noEmit` 检查。

### Security
- TUI 渲染每行强制 `\x1b[0m` 收尾，消除 SGR 颜色泄漏到后续行。

### Test
- `bun test packages`: **124 pass / 0 fail**
  - 新增 `packages/tui/src/render.test.ts`：9 例渲染原语单测
  - 新增 `packages/tui/src/viewport.test.ts`：10 例 viewport / 鼠标解析单测
- `bun run lint`: 0 error
- `bun run compile` + 体积门禁: **61 MB ≤ 61 MB PASS**

[Unreleased]: https://github.com/Agions/vynth/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/Agions/vynth/releases/tag/v0.1.1
[0.1.0]: https://github.com/Agions/vynth/releases/tag/v0.1.0

> **统一发布**：本次发布合并 v0.1.0（初版骨架）/ v0.2.0（MVP 闭环）/
> v0.2.1（错误码 6 位化 + demo 移除 + 模型回滚 + DeepSeek V4 thinking）三段
> 历史到单一 `v0.1.0` 版本。GitHub tag `v0.1.0` 接管同名历史 tag。

### Added
- **错误码 6 位体系（VC-XXXXXX）**：`packages/core/src/error-codes.ts`
  权威表（22 个已声明码 + 6 个族默认回退码）+ `isVynthErrorCode` / `fromLegacy`
  / `describe` 工具函数；`VynthError` 基类新增 `numericCode: VynthErrorCode`
  字段，旧 `code: string` 保留向后兼容。
- `packages/core/src/error-codes.test.ts`：8 例单测覆盖合法性、漂移防护、
  旧字符串解码。
- `packages/sandbox/src/sandbox.ts`：`formatErr()` 自动提取 `numericCode`
  加 `[VC-XXXXXX]` 前缀（网络阻断 / 超时 / 非 0 exit 三类）。
- `apps/cli/src/main.ts`：CLI 参数解析错误带 `[VC-010002]` / `[VC-010003]` /
  `[VC-010004]` 前缀；顶层 `try/catch` + `formatErr()` 输出 6 位码格式。
- `packages/harness/src/e2e.test.ts`：CLI 退出码契约新增 6 位码正则断言；
  空 `VYNTH_API_KEY` → exit 1 + `[VC-XXXXXX] missing VYNTH_API_KEY` 断言。
- **Agent 引擎 + LLM（F4 / F6 / F7 / F8）**：`@vynth/engine` 提供 `runAgent`
  单步上限循环（`maxSteps`，默认 8）、`OpenAiProvider` 解析 OpenAI 兼容 SSE
  （含 `tool_calls` 与顶层 `usage`）；`StreamEvent` 新增 `reasoning` 类型，
  解析 DeepSeek V4 `delta.reasoning_content`。
- **DeepSeek V4 thinking 模式支持**：`ChatMessage` 新增 `reasoning_content` /
  `tool_calls` / `tool_call_id` 字段；`ToolCall` 新增 `rawArgs` 原始 JSON 字符串；
  agent loop 累积 `reasoningContent` 并在 assistant 消息中回传（避免 DeepSeek
  V4 thinking 模式工具调用 400 错误）；tool 消息改用 `tool_call_id` 替代 `name`
  满足 OpenAI 格式；非 2xx 响应时读取响应体并附在错误消息中（调试可见性）。
- **内置工具 + 沙箱守卫（F5 / F10）**：`@vynth/sandbox` 的 `safeResolve`
  拒绝 `../` 越界、cwd 外绝对路径，以及对 realpath 后的 symlink 逃逸；
  `run_shell` 受 `VYNTH_NET='0'` 阻断联网；`runCommand` / `readText` /
  `writeText` 全部经统一守卫。
- **插件无头接入（F9）**：`@vynth/plugins` 的 `loadPlugin` 动态 `import()`
  加载并校验 `pluginName` / `activate`；`loadAll` 批量激活并返回已加载名；
  6 例包级单测覆盖合法加载、批量加载、缺导出与无法 import 四类异常路径。
- **TUI 双模式契约（F2 / F3）**：`@vynth/tui` 的 `startTui` 与 CLI `runHeadless`
  分别承担交互 TUI 与无头模式；非 TTY 守卫——非交互终端无 `-g` 时退出码 `2`
  并提示改用无头模式；headless 不依赖 TTY，可在管道 / CI 使用；Catppuccin
  mocha/latte 主题。
- **工程规矩闸门（CI 8 阶段）**：`.github/workflows/ci.yml` 涵盖 install →
  lint → build → compile → test → gitleaks → 体积门禁 → sign/publish
  （仅 tag）；`gitleaks.toml` 锁定密钥硬编码红线；`scripts/check-binary-size.ts`
  单二进制体积门禁（MVP ≤ 61 MB）；`docs/development/dev-guide.md` 统一分支
  模型、Conventional Commits、错误码规范、安全红线、冻结值表。
- **冷启动基线测量**：`scripts/bench-cold-start.ts` 输出 P50 / P95 / max，
  实测 **P95 ≤ 150 ms 基线**。
- **CLI 退出码契约（F11）**：未知参数 / `-g` 缺值 / `-m` 非法值统一退出码 `2`
  并 stderr 提示（带 `[VC-XXXXXX]` 前缀）；`--version` / `--help` 退出码 `0`；
  空 goal 在非 TTY 退出码 `2`。
- **MIT License**：根目录 `LICENSE` 文件，版权归 Agions (2026)。
- **架构交付归档**：`delivery/` 收纳 G1–G6 七道阶段门的全部产物
  （material_digest / research_report / 高层架构 / 系统设计 / UserStory /
  部署设计 / 安全设计 / G6_交付汇总）。
- **实施开发计划**：`docs/实施开发计划.md` 串联 G0 启动 → G6 交付 → Sprint
  1–6 实施，形成可执行路线图。
- **文档体系 redesign**：`README.md` + `docs/` 多层级结构
  （`guide/` / `architecture/` / `api/` / `development/` / `faq/` /
  `changelog/`），三档阅读路径（新手 15 分钟 / 进阶 30 分钟 / 深度 2 小时+）。
- 首次发布骨架：`@vynth/core` / `engine` / `sandbox` / `tui` / `plugins` /
  `mcp` / `harness` + `apps/cli`。无头 Agent 模式 (`vynth -g "<目标>"`)、
  OpenAI 兼容 LLM、`read_file` / `write_file` / `run_shell` 内置工具。

### Changed
- `apps/cli` 与所有 `packages/*` 同步至 **`0.1.0`**。
- 默认模型 **`deepseek-v4-pro`**（用户最新声明，与冻结裁决 X1/X2 一致）；
  默认端点 `https://api.deepseek.com/v1`（OpenAI 兼容）。
- `docs/guide/getting-started.md` 体积与冷启动描述由历史值改为实测值
  （**60.51 MB** / **P95 ≤ 150 ms**）。
- CLI `--help` 默认模型描述改为 `deepseek-v4-pro`；CLI 参数解析错误带
  `[VC-XXXXXX]` 前缀。
- 文档结构：`docs/dev-guide.md` → `docs/development/dev-guide.md`；
  `docs/release-notes-v0.*.md` → `docs/changelog/v0.*.md`；新增
  `docs/api/reference.md`、`docs/architecture/data-flow.md`、
  `docs/guide/configuration.md`、`docs/guide/plugins.md`、
  `docs/development/contributing.md`、`docs/development/testing.md`、
  `docs/faq/index.md`、`docs/changelog/index.md`。

### Removed（**Breaking**）
- **`EchoProvider` 类与 demo 模式已删除**：`createProvider(config)` 在
  `apiKey` 为空时**抛 `LlmError`**，不再静默 fallback。`VYNTH_API_KEY` 为
  必填项。所有用户须先配置 API Key 才能运行 Vynth。
- `scripts/bench-cold-start.ts` 改为注入 fake key 走真实 CLI 路径。
- GitHub 上游清理：删除孤儿 tag `0.0.1` / `spec-a-architecture-baseline` /
  `v0.1.1` / `v0.2.2` / `v0.2.3`；删除 synerix 时代 stale release；
  接管同名孤儿 tag `v0.1.0`（已存在）。

### Security
- 错误字符串前缀化：`[VC-XXXXXX] message` 可被 grep / 日志聚合 / 监控告警
  按码字符串直接定位；错误码权威化降低误读风险。
- **CLI 退出码契约**：用法错误统一退出码 `2`（带 6 位码前缀），显式拒绝
  未知参数 / 缺值 / 非法值。
- `OpenAiProvider` 拒绝向远程明文 `http` 端点发送 API Key（localhost 放行）；
  非默认端点告警。
- 沙箱守卫覆盖 `..` 越界、绝对路径越界、symlink 逃逸三类逃逸路径
  （**F10 对抗 X3** 单测 100% 拦截），并经 `VYNTH_NET='0'` 阻断 `run_shell`
  联网。
- gitleaks 密钥扫描入 CI 红线；体积门禁防回归。

### Test
- `bun test packages`: **69 pass / 0 fail**
  - `@vynth/core`: 21 例（loadConfig 默认值、VYNTH_NET 解析、错误码层级、
    Emitter、log 分级、error-codes 8 例）
  - `apps/cli` e2e / `@vynth/harness`: 12 例（agent 流式工具、SSE 解析、
    plugin、EchoProvider、sandbox 守卫、symlink、联网拒止、明文 http 拒绝、
    CLI 退出码契约含 6 位码断言、CLI TUI 分流、空 API_KEY 抛 LlmError）
  - `@vynth/engine`: 8 例（无 Key 抛 LlmError、runAgent 流式与终止、maxSteps、
    OpenAI SSE token+tool_calls+usage、reasoning_content 解析、400 错误体读取、
    tool_call_id 消息构建、http 拒绝）
  - `@vynth/sandbox`: 7 例（cwd 内可读、../、绝对路径、symlink、writeText
    越界、VYNTH_NET 开/关）
  - `@vynth/plugins`: 6 例（合法加载、批量加载、三类异常路径）
  - `@vynth/tui/theme`: 5 例（mocha/latte 色值、fg/bg/reset）
  - `@vynth/tui/stream-escape-hatch`: 5 例（首次不擦线、二次擦、clear 归零、
    空 clear、空串）
- `bun run lint`: 0 error（59 文件）
- `bun run compile` + 体积门禁: **60.51 MB < 61 MB PASS**
- `bun scripts/bench-cold-start.ts`: P95 ≤ 150 ms PASS

[Unreleased]: https://github.com/Agions/vynth/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Agions/vynth/releases/tag/v0.1.0