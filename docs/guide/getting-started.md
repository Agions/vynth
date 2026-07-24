# 快速开始（30 秒跑通）

Vynth 是「你 terminal 里的代码合成器」——纯 TypeScript 全量构建，单二进制分发。无需 API key 即可体验**流式输出与工具循环（demo 模式）**。

## 1. 安装依赖

```bash
bun install        # 推荐（脚本均使用 bun）；或 pnpm install（pnpm-workspace.yaml 已配置）
```

- 运行时要求：**Bun >= 1.1**（`package.json` 的 `engines`）。
- 包管理：pnpm workspace + turbo。

## 2. 编译单二进制

```bash
bun run compile     # → 产出 dist/vynth（bun build --compile）
```

> 当前二进制体积约 61MB（含 react-devtools 残留），目标收敛到 20–40MB（优化进行中）。冷启动目标 50–150ms。

## 3. 无 key 先跑 demo（无需 API key）

```bash
./dist/vynth -g '用一句话介绍 vynth'
```

未设置 `VYNTH_API_KEY` 时自动进入 **demo（EchoProvider）** 模式：离线流式输出 +（goal 含 `demo-tool` 时）工具调用演示。

## 4. 接入真实 LLM

设置环境变量后再次运行，即可接入 OpenAI 兼容端点：

```bash
export VYNTH_API_KEY="sk-..."                 # 必填：真实 LLM key
export VYNTH_MODEL="gpt-4o-mini"              # 可选：默认 gpt-4o-mini
export VYNTH_LLM_BASE_URL="https://api.openai.com/v1"  # 可选：OpenAI 兼容端点
export VYNTH_MODE="vibe"                      # 可选：plan | vibe（默认 vibe）

./dist/vynth -g '给当前目录写一份 README.md'
```

- `VYNTH_API_KEY` 为空 → demo；非空 → `OpenAiProvider` 走 SSE 流式。
- 想用本地 / 第三方兼容服务，改 `VYNTH_LLM_BASE_URL` 即可（如 `http://localhost:11434/v1`）。

## 5. 启动交互 TUI

```bash
./dist/vynth        # 需真实 TTY（raw mode）；无 TTY 时退化为普通输入
```

TUI 为轻量 ANSI 界面（**非 ink**），逐字符流式 + 工具调用回显；`Ctrl-C` / `Ctrl-D` 退出。

## 6. 加载插件与信任边界

插件经动态 `import()` 直接加载并执行，能力等同 Vynth 本体——**请勿加载来源不明的插件**。

```bash
./dist/vynth -g '用 hello 工具向世界问好' -p packages/plugins/examples/hello-plugin.ts
```

插件入口需 `export const pluginName` 与 `export function activate(reg)`（参考 `packages/plugins/examples/hello-plugin.ts`）。

> ⚠ **信任边界（必读）**：`-p` 加载的插件在当前进程中执行**任意代码**，拥有与 Vynth 同等的文件系统、环境变量（含 `VYNTH_API_KEY`）、网络与命令执行权限。**Vynth 不对插件代码做沙箱隔离**。仅从你完全信任的来源加载插件；恶意插件可窃取凭据或破坏系统。内置 `read_file`/`write_file` 受 cwd 越界守卫约束；`run_shell` 以**宿主权限**运行（`sh -c`，仅设工作目录、无隔离）；插件注册的自定义工具同样不受沙箱约束。另：联网开关（`VYNTH_NET='0'`）为尽力而为、且对 `run_shell` 当前不生效，非硬安全边界。

## 环境变量速查

| 变量 | 作用 | 默认 |
|------|------|------|
| `VYNTH_API_KEY` | LLM key（空 = demo） | 空 |
| `VYNTH_MODEL` | 模型名 | `gpt-4o-mini` |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.openai.com/v1` |
| `VYNTH_MODE` | `plan` \| `vibe` | `vibe` |
| `VYNTH_THEME` | `mocha` \| `latte`（Catppuccin） | `mocha` |
| `VYNTH_NET` | 沙箱网络开关（`'0'` = 禁网） | 开启 |
| `VYNTH_DATA_DIR` | 数据目录 | `~/.vynth` |

> 完整 CLI 入口与退出码见 [API 总览](../api/overview.md)；架构与包职责见 [架构总览](../architecture/index.md)。

## 常见问题

- **`vynth` 命令找不到**：用 `./dist/vynth`，或 `bun link` / 加入 `PATH`。
- **TUI 卡住 / 无响应**：确认在真实终端（TTY）中运行；管道 / CI 环境请用 `-g` 无头模式。
- **想体验工具调用**：demo 下用 `./dist/vynth -g 'demo-tool 请调用示例工具'` 触发内置 `EchoProvider` 的工具演示链路。
