# 快速开始（30 秒跑通）

Vynth 是「你 terminal 里的代码合成器」——纯 TypeScript 全量构建，单二进制分发。设置 `VYNTH_API_KEY` 即可接入真实 LLM。

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

> 实测：单二进制 **60.51 MB**（无 react-devtools 残留，TUI 走轻量 ANSI 不依赖 ink）；冷启动 **P95 = 30.5 ms**（10 次采样，远低于 150 ms 基线）。

## 3. 接入真实 LLM

设置环境变量后运行：

```bash
export VYNTH_API_KEY="sk-..."                 # 必填：真实 LLM key
export VYNTH_MODEL="deepseek-v4-pro"             # 可选：默认 deepseek-v4-pro
export VYNTH_LLM_BASE_URL="https://api.deepseek.com/v1"  # 可选：DeepSeek 兼容端点
export VYNTH_MODE="vibe"                      # 可选：plan | vibe（默认 vibe）

./dist/vynth -g '给当前目录写一份 README.md'
```

- `VYNTH_API_KEY` 为空将抛出 `LlmError`，不会进入 demo 模式。
- 想用本地 / 第三方兼容服务，改 `VYNTH_LLM_BASE_URL` 即可（如 `http://localhost:11434/v1`）。

## 4. 启动交互 TUI

```bash
./dist/vynth        # 需真实 TTY（raw mode）；无 TTY 时退化为普通输入
```

TUI 为轻量 ANSI 界面（**非 ink**），逐字符流式 + 工具调用回显；`Ctrl-C` / `Ctrl-D` 退出。

## 5. 加载插件与信任边界

插件经动态 `import()` 直接加载并执行，能力等同 Vynth 本体——**请勿加载来源不明的插件**。

```bash
./dist/vynth -g '用 hello 工具向世界问好' -p packages/plugins/examples/hello-plugin.ts
```

插件入口需 `export const pluginName` 与 `export function activate(reg)`（参考 `packages/plugins/examples/hello-plugin.ts`）。

> ⚠ **信任边界（必读）**：`-p` 加载的插件在当前进程中执行**任意代码**，拥有与 Vynth 同等的文件系统、环境变量（含 `VYNTH_API_KEY`）、网络与命令执行权限。**Vynth 不对插件代码做沙箱隔离**。仅从你完全信任的来源加载插件；恶意插件可窃取凭据或破坏系统。内置 `read_file`/`write_file` 受 cwd 越界守卫约束；`run_shell` 以**宿主权限**运行（`sh -c`，仅设工作目录、无隔离）；插件注册的自定义工具同样不受沙箱约束。另：联网开关（`VYNTH_NET='0'`）现已生效，`run_shell` 受其为阻断；网络隔离仍为尽力而为、非硬安全边界。

## 环境变量速查

| 变量 | 作用 | 默认 |
|------|------|------|
| `VYNTH_API_KEY` | LLM key（必填） | 空 |
| `VYNTH_MODEL` | 模型名（默认已指向 DeepSeek 最新通用模型） | `deepseek-v4-pro` |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点（默认已指向 DeepSeek） | `https://api.deepseek.com/v1` |
| `VYNTH_MODE` | `plan` \| `vibe` | `vibe` |
| `VYNTH_THEME` | `mocha` \| `latte`（Catppuccin） | `mocha` |
| `VYNTH_NET` | 沙箱网络开关（`'0'` = 禁网） | 开启 |
| `VYNTH_DATA_DIR` | 数据目录 | `~/.vynth` |

> 完整 CLI 入口与退出码见 [API 总览](../api/overview.md)；架构与包职责见 [架构总览](../architecture/index.md)。

## 常见问题

- **`vynth` 命令找不到**：��� `./dist/vynth`，或 `bun link` / 加入 `PATH`。
- **TUI 卡住 / 无响应**：确认在真实终端（TTY）中运行；管道 / CI 环境请用 `-g` 无头模式。
- **想体验工具调用**：确保已设置 `VYNTH_API_KEY`，使用 `./dist/vynth -g '你的目标'` 触发真实 LLM 的工具调用链路。
