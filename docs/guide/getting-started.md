# 快速开始

从零到第一次 AI 改代码，5 分钟。

## 1. 安装

```bash
# macOS / Linux 一行命令
curl -fsSL https://raw.githubusercontent.com/Agions/vynth/main/scripts/install.sh | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/Agions/vynth/main/scripts/install.ps1 | iex
```

脚本自动处理 Bun 环境、编译与 PATH。其他方式（源码构建 / 二进制分发）见 [安装指南](installation.md)。

```bash
zeno --version    # 0.1.1 → 安装成功
```

## 2. 配置 API Key

```bash
export ZENO_API_KEY="sk-..."    # 必填，唯一的强制配置
```

默认接 DeepSeek（`deepseek-v4-pro`）。换模型 / 端点任选一种方式：

```bash
# 方式 A：环境变量
export ZENO_MODEL="gpt-5.6-terra"
export ZENO_LLM_BASE_URL="https://api.openai.com/v1"

# 方式 B：TUI 内配置（推荐，写入 ~/.zeno/config.json）
zeno            # 进入 TUI 后输入 /model gpt-5.6-terra https://api.openai.com/v1
                # 或 /config 打开可视化配置面板

# 方式 C：本地模型
export ZENO_LLM_BASE_URL="http://localhost:11434/v1"
```

> `ZENO_API_KEY` 为空会直接报错，不会进入 demo 模式。API Key 只走环境变量或 TUI 内配置，**不要**手写进项目配置文件。

## 3. 第一次运行

### 交互式 TUI（推荐先体验）

```bash
zeno
```

进入全屏界面后试试：

- 直接输入目标：`给 src/utils 补全 JSDoc 注释`
- `Tab`（空输入时）切换 **Vibe**（边聊边写）/ **Plan**（先规划再执行）
- `/` 呼出命令浮窗，`@` 引用工作区文件
- `Ctrl+B` 开侧栏（文件 / 任务 / 工具），`Ctrl+O` 开分屏看工具输出流

完整快捷键与斜杠命令见 [TUI 使用指南](tui.md)。

### 无头模式（脚本 / CI）

```bash
zeno -g '给当前目录写一份 README.md'          # 流式输出到 stdout
zeno -m plan -g '实现用户认证模块'             # 指定 plan 模式
```

## 4. 进阶能力

### 加载插件

```bash
zeno -g '用 hello 工具向世界问好' -p packages/plugins/examples/hello-plugin.ts
```

> ⚠️ **信任边界（必读）**：插件在 Zeno 进程内执行任意代码，拥有与 Zeno 同等的文件、环境变量（含 `ZENO_API_KEY`）、网络与命令执行权限，**不做沙箱隔离**。仅加载完全信任的插件。TUI 模式下加载会弹出信任确认；无头模式 `-p` 即视为授权。

### 接入 MCP Server

```bash
zeno -g '查询天气' -s "npx -y @modelcontextprotocol/server-xxx"    # 可重复 -s 接多个
```

MCP 工具自动并入 agent 工具集，走同一套沙箱 / 审计链路。

### 安全加固

```bash
export ZENO_NET=0        # 关闭联网
export ZENO_HARDEN=1     # OS 级硬隔离（Linux bubblewrap / macOS seatbelt，见安装指南平台矩阵）
export ZENO_AUDIT=1      # 5 维审计日志
```

## 环境变量速查

| 变量 | 作用 | 默认 |
| --- | --- | --- |
| `ZENO_API_KEY` | LLM API Key（**必填**） | — |
| `ZENO_MODEL` | 模型名 | `deepseek-v4-pro` |
| `ZENO_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` |
| `ZENO_MODE` | `plan` \| `vibe` | `vibe` |
| `ZENO_THEME` | `mocha` / `latte` / `midnight` / `forest` / `light` / `neon` | `mocha` |
| `ZENO_NET` | 联网开关（`0` = 禁网） | 开启 |
| `ZENO_HARDEN` | OS 级硬隔离（`1` = 开启，不可用时 Fail-Closed） | 关闭 |
| `ZENO_AUDIT` | 审计日志（`1` = 开启） | 关闭 |
| `ZENO_DATA_DIR` | 数据目录 | `~/.zeno` |
| `ZENO_REPOMAP` | 仓库地图（`0` = 关闭） | 开启 |

完整优先级规则（CLI 参数 > 环境变量 > 配置文件）见 [配置详解](configuration.md)。

## 常见问题

- **`zeno` 找不到**：安装目录不在 PATH——重开终端，或按安装脚本结尾的提示追加 PATH。
- **TUI 卡住 / 无响应**：确认在真实终端（TTY）运行；管道 / CI 用 `-g` 无头模式。
- **改了配置没生效**：环境变量优先级高于配置文件，先 `env | grep ZENO` 排查覆盖。

下一步：[TUI 使用指南](tui.md) · [配置详解](configuration.md) · [FAQ](../faq/index.md)
