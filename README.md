# Vynth

> 你 terminal 里的代码合成器（AI-Native Coding Terminal）

Vynth 是一个终端 TUI 的 Vibe Coding 工具，支持 **Plan** / **Vibe** 双模式，把自然语言"合成"成代码。

## 架构（纯 TypeScript 全量构建）

- **运行时**：Bun + `bun build --compile` 单二进制
- **TUI**：自研轻量 ANSI 渲染器（非 ink）+ 流式直写逃生舱
- **结构**：pnpm workspace + turbo，包以 `@vynth/*` 组织

```
packages/
  core/     共享类型 / 配置 / 错误 / 事件总线 / 日志
  engine/   LLM 客户端 + 工具系统 + agent 循环（llm+tools+agent 合并）
  tui/      轻量 ANSI 渲染器（非 ink）+ Catppuccin 主题 + 流式逃生舱
  sandbox/  fs 路径越界守卫（可被符号链接绕过）；run_shell 以宿主权限运行、无隔离
  mcp/      MCP 客户端（stdio JSON-RPC）
  plugins/  插件加载 / 生命周期
  harness/  集成测试 / e2e 驱动
apps/
  cli/      CLI 入口（bin: vynth）
```

## 快速开始

```bash
bun install            # 或 pnpm install
bun run compile        # 产出 dist/vynth 单二进制
./dist/vynth --help
./dist/vynth --goal "给当前目录写个 README"   # 无头 agent 模式
./dist/vynth           # 启动 TUI
```

无 `VYNTH_API_KEY` 时自动进入 **demo（echo）provider**，可离线体验流式与工具循环。

## 配置

通过环境变量配置（不读取任何配置文件）：

- `VYNTH_LLM_BASE_URL`：OpenAI 兼容端点（默认 `https://api.openai.com/v1`）
- `VYNTH_API_KEY`：API key
- `VYNTH_MODEL`：模型名（默认 `gpt-4o-mini`）
- `VYNTH_MODE`：`plan` | `vibe`（默认 `vibe`）
