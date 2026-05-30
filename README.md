# Syncode

> AI 配对编程终端 — 让 AI 与你的代码同步

[![CI](https://gitee.com/Agions/syncode/badges/master/pipeline.svg)](https://gitee.com/Agions/syncode/pipelines)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![npm](https://img.shields.io/npm/v/syncode.svg)](https://www.npmjs.com/package/syncode)
[![Install](https://img.shields.io/badge/install-curl%20%7C%20npm%20%7C%20cargo-brightgreen.svg)](#安装)

一款高性能、单进程 TUI 应用，融合了 Claude Code 的交互模型、Codex CLI 的沙箱机制和 OpenCode 的可扩展架构。

## 功能特性

| 功能 | 说明 |
|------|------|
| **多智能体** | Agent Swarm 架构，Coder/Reviewer/Tester/Architect 角色 |
| **工作流引擎** | YAML 定义多步骤流水线，依赖/分支/变量插值 |
| **智能体循环** | 流式推理 → 工具分发 → 多轮推理 |
| **LLM 集成** | DeepSeek V4、MiMo-v2.5 及任意 OpenAI 兼容 API |
| **5 个内置工具** | 文件读写、Shell 执行、搜索 (ripgrep)、补丁应用 |
| **TUI 界面** | ratatui 五区布局，Tokyo Night 主题 |
| **Diff 高亮** | syntect 语法高亮，统一视图 + 并排视图 |
| **Vim/Emacs 键位** | 完整 Vim 模式编辑 + Emacs 非模式编辑 |
| **鼠标支持** | 点击聚焦、滚轮滚动、侧边栏标签切换 |
| **技能系统** | YAML/MD 技能文件，自动匹配加载 |
| **MCP 协议** | 原生客户端，支持 stdio/HTTP 传输 |
| **沙箱安全** | 命令风险分级、原子写入、审批流程 |
| **配置热重载** | mtime 轮询 + SIGHUP 信号 |
| **会话持久化** | SQLite (WAL 模式) 完整 CRUD |

## 安装

### curl 一键安装 (推荐)

```bash
curl -fsSL https://gitee.com/Agions/syncode/raw/main/install.sh | bash
```

支持 Linux (x86_64/aarch64) 和 macOS (x86_64/arm64)。自动检测平台，下载预编译二进制，若无预编译版本则自动从源码构建。

自定义安装目录：

```bash
INSTALL_DIR=~/.local/bin curl -fsSL https://gitee.com/Agions/syncode/raw/main/install.sh | bash
```

### npm 安装

```bash
# 全局安装
npm install -g syncode

# 或直接运行（无需安装）
npx syncode
```

支持 Linux (x86_64/aarch64) 和 macOS (x86_64/arm64)。自动下载预编译二进制。

### Cargo 安装

```bash
# 从源码安装
cargo install --path .

# 或本地构建
cargo build --release
./target/release/syncode
```

## 卸载

### curl 一键卸载

```bash
# 卸载二进制（保留配置）
curl -fsSL https://gitee.com/Agions/syncode/raw/main/uninstall.sh | bash

# 完全卸载（包括配置和数据）
curl -fsSL https://gitee.com/Agions/syncode/raw/main/uninstall.sh | bash -s -- --all

# 仅删除配置
curl -fsSL https://gitee.com/Agions/syncode/raw/main/uninstall.sh | bash -s -- --config-only
```

### npm 卸载

```bash
npm uninstall -g syncode
```

### Cargo 卸载

```bash
cargo uninstall syncode
```

### 手动清理

```bash
# 删除二进制
rm -f /usr/local/bin/syncode
rm -f ~/.local/bin/syncode

# 删除配置目录
rm -rf ~/.config/syncode/

# 删除缓存和数据
rm -rf ~/.cache/syncode/
rm -rf ~/.local/share/syncode/
```

## 配置

配置文件路径：`~/.config/syncode/config.toml`

```toml
[llm]
provider = "deepseek"       # deepseek | mimo | custom
api_key = "your-api-key"
model = "deepseek-chat"
context_window = 128000
max_output_tokens = 8192

[ui]
theme = "dark"              # dark | light
keymap = "default"          # vim | emacs | default

[sandbox]
mode = "confirm"            # auto | confirm | preview_only
atomic_writes = true
```

环境变量覆盖：
- `SYNCODE_API_KEY` — LLM API 密钥
- `SYNCODE_BASE_URL` — API 基础 URL
- `SYNCODE_MODEL` — 模型标识符

### MCP 服务器配置

在 `config.toml` 中配置 MCP 服务器：

```toml
# stdio 传输（本地进程）
[[mcp]]
name = "gitee"
type = "stdio"
command = "npx"
args = ["-y", "@gitee/mcp-gitee"]
auto_reconnect = true
timeout_secs = 30

[mcp.env]
GITEE_TOKEN = "your-token"

# HTTP 传输（远程服务器）
[[mcp]]
name = "remote-tools"
type = "http"
url = "https://mcp.example.com/sse"
timeout_secs = 60
```

### 外部技能来源

支持从多个位置加载技能：

```toml
# 本地目录
[[skill_sources]]
type = "local"
location = "~/.config/syncode/skills"

# Git 仓库（自动克隆+更新）
[[skill_sources]]
type = "git"
location = "https://gitee.com/Agions/syncode-skills.git"
branch = "main"
include = ["**/*.md"]

# 远程文件
[[skill_sources]]
type = "url"
location = "https://example.com/skills/code-review.md"
```

### 自定义智能体

支持两种方式定义自定义智能体：

**方式一：内联定义（config.toml）**

```toml
[[agents]]
name = "security-auditor"
description = "安全审计专家"
system_prompt = "你是安全审计专家，专注于漏洞检测..."
tools = ["file_read", "search"]
max_turns = 8
tags = ["security"]
```

**方式二：YAML 文件（~/.config/syncode/agents/*.yaml）**

```yaml
name: performance-analyst
description: 性能优化专家
system_prompt: |
  你是性能优化专家，专注于算法复杂度、
  内存分配、数据库查询优化...
tools:
  - file_read
  - search
  - shell_exec
max_turns: 12
capabilities:
  can_write_code: true
  can_run_tests: true
tags:
  - performance
```

## 多智能体与工作流

### 内置智能体角色

| 角色 | 职责 | 能力 |
|------|------|------|
| **Coder** | 编写和修改代码 | 读写文件、执行命令、运行测试 |
| **Reviewer** | 代码审查 | 只读文件、搜索代码 |
| **Tester** | 编写和运行测试 | 读写文件、执行命令 |
| **Architect** | 架构设计 | 只读文件、搜索代码 |
| **Planner** | 任务分解和规划 | 无工具权限 |

### 内置工作流

| 工作流 | 流程 | 说明 |
|--------|------|------|
| **code-review** | Coder → Reviewer → Tester | 自动化代码审查 |
| **refactor** | Architect → Coder → Reviewer | 结构化重构 |
| **debug** | Tester → Coder → Tester | 系统化调试 |

### 工作流 YAML 格式

```yaml
name: my-workflow
description: 自定义工作流
version: "1.0"

variables:
  language: Rust

steps:
  - id: step1
    agent_role: coder
    prompt: "用 {{language}} 实现：{{task}}"
    output_variable: code
    timeout_secs: 300

  - id: step2
    agent_role: reviewer
    prompt: "审查代码：{{code}}"
    depends_on: [step1]
    condition: code
    output_variable: feedback
```

### 工作流执行

```bash
# 使用内置工作流
syncode --workflow workflows/code-review.yaml --var task="实现用户登录"

# 使用自定义工作流
syncode --workflow my-workflow.yaml --var task="重构认证模块"
```

## 键位映射

### Vim 模式

| 模式 | 按键 | 功能 |
|------|------|------|
| 普通 | `i` / `a` / `A` | 进入插入模式 |
| 普通 | `:` / `/` | 命令模式 / 搜索模式 |
| 普通 | `j` / `k` | 向下 / 向上滚动 |
| 普通 | `G` | 滚动到底部 |
| 普通 | `dd` | 清除当前行 |
| 普通 | `yy` / `p` | 复制 / 粘贴 |
| 插入 | `Esc` | 返回普通模式 |
| 插入 | `Ctrl+w` | 删除单词 |
| 插入 | `Ctrl+k` / `Ctrl+u` | 删除到行尾 / 行首 |

### Emacs 模式

| 按键 | 功能 |
|------|------|
| `Ctrl+n` / `Ctrl+p` | 向下 / 向上滚动 |
| `Ctrl+f` / `Ctrl+b` | 光标右移 / 左移 |
| `Ctrl+a` / `Ctrl+e` | 跳到行首 / 行尾 |
| `Ctrl+k` / `Ctrl+y` | 删除 / 粘贴 |

## 架构

```
src/
├── main.rs           # 入口 + 启动计时
├── app.rs            # 应用状态机 + 事件分发
├── error.rs          # 统一错误类型 (thiserror)
├── lib.rs            # 库 crate 导出
├── telemetry.rs      # 启动指标
├── config/
│   ├── settings.rs   # TOML 配置 + 环境变量覆盖
│   ├── keymap.rs     # Vim/Emacs 键位配置
│   └── watcher.rs    # 配置热重载 (mtime + SIGHUP)
├── tui/
│   ├── theme.rs      # Tokyo Night 亮暗主题
│   ├── frame.rs      # 五区布局渲染
│   ├── diff_renderer.rs  # syntect Diff 高亮
│   ├── syntax.rs     # 代码高亮引擎
│   ├── event.rs      # crossterm 事件源
│   └── widgets/      # 7 个可组合 UI 组件
├── llm/
│   ├── adapter.rs    # LLM 适配器 trait + OpenAI 兼容
│   ├── stream.rs     # SSE 流解析器
│   └── types.rs      # 统一 LLM 类型
├── agent/
│   ├── agloop.rs     # 核心智能体循环
│   ├── context.rs    # Token 预算 + 动态裁剪
│   └── prompt.rs     # 系统提示词构建
├── tools/
│   ├── registry.rs   # 工具注册中心
│   └── builtin/      # 5 个内置工具
├── skills/
│   ├── registry.rs   # 技能注册中心
│   ├── loader.rs     # YAML frontmatter 解析
│   └── builtin/      # 内置技能
├── mcp/
│   ├── client.rs     # MCP 客户端 (stdio)
│   ├── manager.rs    # 多服务器管理器
│   └── transport.rs  # 传输 trait
├── sandbox/
│   ├── command_preview.rs  # 风险分级
│   ├── atomic_replace.rs   # 崩溃安全写入
│   └── approval.rs         # 审批流程
└── session/
    ├── store.rs      # SQLite 持久化
    └── model.rs      # 会话/消息模型
```

## 技术栈

| 组件 | 选型 | 版本 |
|------|------|------|
| 语言 | Rust | 1.75+ |
| TUI | ratatui + crossterm | 0.28 |
| 异步运行时 | tokio | 1.x |
| HTTP 客户端 | reqwest | 0.12 |
| 数据库 | rusqlite (bundled) | 0.31 |
| 语法高亮 | syntect | 5.x |
| 错误处理 | thiserror | 2.x |
| 配置解析 | toml | 0.8 |
| 日志 | tracing | 0.1 |

## 测试

```bash
# 运行全部测试 (115 个测试)
cargo test

# 运行特定测试套件
cargo test --test e2e          # 端到端测试 (Mock LLM)
cargo test --test phase2       # 工具 + 沙箱
cargo test --test phase3       # 主题 + Diff + 语法高亮
cargo test --test phase4       # 键位 + 鼠标

# 启动性能基准测试
cargo run --features startup_bench
```

## 开发

```bash
# 类型检查
cargo check

# 格式化
cargo fmt

# 静态分析
cargo clippy -- -D warnings -A dead_code

# Release 构建
cargo build --release
```

## 开源协议

MIT
