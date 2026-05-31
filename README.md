<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/banner.svg">
    <img src="assets/banner.svg" alt="Synerix — AI-Native Coding Terminal" width="100%">
  </picture>
</p>

<h1 align="center">Synerix</h1>
<p align="center">
  <strong>AI-Native Coding Terminal</strong><br>
  <sub>用 Rust 编写 · 为速度而生 · 多智能体协作 · 安全沙箱</sub>
</p>

<p align="center">
  <a href="https://github.com/Agions/synerix/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Agions/synerix/ci.yml?branch=main&style=flat-square&label=CI&logo=github" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-00D4FF.svg?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75%2B-FF6B3D.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust"></a>
  <img src="https://img.shields.io/badge/Startup-2ms-FF6B9D.svg?style=flat-square" alt="Startup">
  <img src="https://img.shields.io/badge/Binary-3.8_MB-7B61FF.svg?style=flat-square" alt="Size">
  <img src="https://img.shields.io/badge/Tests-1149-00FF88.svg?style=flat-square" alt="Tests">
  <img src="https://img.shields.io/badge/Unsafe-0-FF4444.svg?style=flat-square" alt="Unsafe">
</p>

<p align="center">
  <a href="#-installation">Installation</a> ·
  <a href="#-quick-start">Quick Start</a> ·
  <a href="#-features">Features</a> ·
  <a href="#-configuration">Configuration</a> ·
  <a href="#-architecture">Architecture</a> ·
  <a href="#-development">Development</a>
</p>

---

## 📥 Installation

<table>
<tr>
<td width="50%">

### GitHub (Global)
```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```
</td>
<td width="50%">

### Gitee (China)
```bash
curl -fsSL https://gitee.com/Agions/synerix/raw/main/install.sh | bash
```
</td>
</tr>
<tr>
<td colspan="2">

Custom path:
```bash
INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```
</td>
</tr>
</table>

<details>
<summary><b>🔧 Build from source</b></summary>

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
cargo build --release
# Binary at target/release/synerix
```
</details>

<details>
<summary><b>🗑️ Uninstall</b></summary>

```bash
# Keep configuration
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash

# Full removal (including config and data)
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash -s -- --all
```
</details>

---

## 🚀 Quick Start

### 1. Configure LLM

```bash
mkdir -p ~/.config/synerix

cat > ~/.config/synerix/config.toml << 'EOF'
[llm]
provider = "deepseek"       # deepseek | mimo | custom
api_key = "your-api-key"
model = "deepseek-v4-flash"
context_window = 128000
max_output_tokens = 8192

[ui]
theme = "dark"              # dark | light
keymap = "default"          # vim | emacs | default

[sandbox]
mode = "confirm"            # auto | confirm | preview_only
atomic_writes = true
EOF
```

### 2. Launch

```bash
synerix
```

### 3. Start Coding

```
❯ Refactor this function to improve readability
```

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🤖 Multi-Agent Collaboration
Agent Swarm architecture with 5 specialized roles:

| Role | Responsibility |
|------|---------------|
| **Coder** | Write and modify code |
| **Reviewer** | Code review |
| **Tester** | Write and run tests |
| **Architect** | Architecture design |
| **Planner** | Task decomposition |

**Pipeline:** Streaming inference → Tool dispatch → Parallel execution → Multi-round reasoning

</td>
<td width="50%">

### 🔧 Workflow Engine
YAML-defined multi-step DAG pipelines:

- Dependency resolution + conditional branching
- Variable interpolation + auto-retry
- Built-in code-review / refactor / debug
- Custom workflows with one-command execution

```bash
synerix --workflow workflows/code-review.yaml
```

```yaml
name: code-review
steps:
  - id: analyze
    agent_role: reviewer
    prompt: "Review code changes"
  - id: report
    agent_role: coder
    prompt: "Apply review feedback"
    depends_on: [analyze]
```

</td>
</tr>
<tr>
<td>

### 🎨 Terminal UI
ratatui-powered 5-zone layout with Tokyo Night theme:

- Streaming typewriter effect + thinking animation
- syntect-powered syntax-highlighted diff views
- **Vim** / **Emacs** / Default keymaps
- Full mouse support

</td>
<td>

### 🔒 Security Sandbox
Multi-layer protection for your code:

| Level | Mode | Description |
|-------|------|-------------|
| ⬜ | `auto` | Execute automatically |
| 🟨 | `confirm` | Review before execution |
| 🟥 | `preview_only` | Preview only, no execution |

- Atomic file writes (crash-safe)
- MCP tool permission isolation
- Command risk classification (safe/moderate/dangerous)

</td>
</tr>
<tr>
<td>

### 🧩 Extensible Architecture
Plugin-driven design for maximum flexibility:

- **Skills** — Auto-detect YAML/MD skill files
- **MCP** — Native stdio/HTTP transport
- **Plugins** — Full lifecycle management
- **Custom Agents** — TOML/YAML-defined agents

</td>
<td>

### ⚡ Blazing Performance
Rust zero-cost abstractions, built for speed:

| Metric | Target | Actual |
|--------|:-----:|:------:|
| Startup | <80ms | **2ms** ✅ |
| Binary | ≤15MB | **3.8MB** ✅ |
| Unsafe code | 0 | **0** ✅ |
| Warnings | 0 | **0** ✅ |

LTO + strip + codegen-units=1
Token budgeting + LRU cache + zero-copy

</td>
</tr>
</table>

---

## ⌨️ Slash Commands

Built-in commands available in the chat input:

| Command | Description |
|---------|-------------|
| `/help` | 显示所有斜杠命令 |
| `/clear` | 清空当前对话 |
| `/model` | 切换 LLM 模型 / 配置自定义提供商：`/model <name>` 或 `/model custom <name> <base-url>` |
| `/reset` | 重置对话状态 |
| `/exit` | 退出 Synerix |
| `/workflow` | 运行工作流（如 `/workflow code-review`）|
| `/mcp` | 管理 MCP 服务器：`/mcp`（列表）、`/mcp list`（列表）、`/mcp show <name>`（详情）、`/mcp add <name> stdio <cmd>`（添加 stdio）、`/mcp add <name> http <url>`（添加 HTTP）、`/mcp remove <name>`（删除）|
| `/skill` | 管理技能：`/skill`（查看）、`/skill dir <path>`（设置目录）、`/skill source list`（列出源）、`/skill source add <type> <location>`（添加源）、`/skill source remove <index>`（删除源）|
| `/config` | 管理配置：`/config show`（显示路径）、`/config save`（保存配置到文件）|

---

## ⌨️ Keymaps

<details>
<summary><b>Vim Mode</b></summary>

| Mode | Key | Action |
|:---:|-----|--------|
| Normal | `i` / `a` / `A` | Enter insert mode |
| Normal | `:` / `/` | Command / Search mode |
| Normal | `j` / `k` | Scroll down / up |
| Normal | `G` / `gg` | Scroll to bottom / top |
| Normal | `dd` / `yy` / `p` | Clear / copy / paste |
| Insert | `Esc` | Return to normal mode |
| Insert | `Ctrl+w` | Delete word |
| Insert | `Ctrl+k` / `Ctrl+u` | Delete to EOL / SOL |

</details>

<details>
<summary><b>Emacs Mode</b></summary>

| Key | Action |
|-----|--------|
| `Ctrl+n` / `Ctrl+p` | Scroll down / up |
| `Ctrl+f` / `Ctrl+b` | Cursor right / left |
| `Ctrl+a` / `Ctrl+e` | Jump to BOL / EOL |
| `Ctrl+k` / `Ctrl+y` | Kill / yank |

</details>

---

## 🤖 Multi-Agent & Workflows

### Built-in Workflows

| Workflow | Pipeline | Description |
|----------|----------|-------------|
| **code-review** | `Coder → Reviewer → Tester` | Automated code review |
| **refactor** | `Architect → Coder → Reviewer` | Structured refactoring |
| **debug** | `Tester → Coder → Tester` | Systematic debugging |

### Custom Workflow

```yaml
name: my-workflow
description: Custom workflow definition
version: "1.0"

variables:
  language: Rust

steps:
  - id: implement
    agent_role: coder
    prompt: "Implement {{task}} in {{language}}"
    output_variable: code
    timeout_secs: 300

  - id: review
    agent_role: reviewer
    prompt: "Review code: {{code}}"
    depends_on: [implement]
    output_variable: feedback
```

---

## 🔌 Configuration

### LLM Providers

| Provider | Default Model | Available Models | Notes |
|----------|---------------|-----------------|-------|
| `deepseek` | deepseek-v4-flash | deepseek-v4-flash, deepseek-v4-pro | Default |
| `mimo` | mimo-v2.5-pro | mimo-v2.5-pro | Xiaomi MiMo |
| `custom` | — | Any | OpenAI-compatible API |

### Environment Overrides

| Variable | Description |
|----------|-------------|
| `SYNERIX_API_KEY` | LLM API key |
| `SYNERIX_BASE_URL` | API base URL |
| `SYNERIX_MODEL` | Model identifier |

### MCP Servers

```toml
# stdio transport (local process)
[[mcp]]
name = "gitee"
type = "stdio"
command = "npx"
args = ["-y", "@gitee/mcp-gitee"]
auto_reconnect = true
timeout_secs = 30

[mcp.env]
GITEE_TOKEN = "your-token"

# HTTP transport (remote server)
[[mcp]]
name = "remote-tools"
type = "http"
url = "https://mcp.example.com/sse"
timeout_secs = 60
```

对应的斜杠命令：

| 操作 | 命令 |
|------|------|
| 查看列表 | `/mcp` 或 `/mcp list` |
| 查看详情 | `/mcp show <name>` |
| 添加 stdio | `/mcp add <name> stdio <command> [args...]` |
| 添加 HTTP | `/mcp add <name> http <url>` |
| 删除 | `/mcp remove <name>` |

### External Skills

```toml
# Local directory
[[skill_sources]]
type = "local"
location = "~/.config/synerix/skills"

# Git repository (auto-cloning + updates)
[[skill_sources]]
type = "git"
location = "https://github.com/Agions/synerix-skills.git"
branch = "main"
include = ["**/*.md"]
```

对应的斜杠命令：

| 操作 | 命令 |
|------|------|
| 查看状态 | `/skill` |
| 设置目录 | `/skill dir <path>` |
| 列出技能源 | `/skill source list` |
| 添加技能源 | `/skill source add <type> <location> [branch]` |
| 删除技能源 | `/skill source remove <index>` |

### Custom Agents

```toml
[[agents]]
name = "security-auditor"
description = "Security audit specialist"
system_prompt = "You are a security audit expert..."
tools = ["file_read", "search"]
max_turns = 8
tags = ["security"]
```

---

## 🏗️ Architecture

```
src/
├── main.rs                 # Entry point + startup timing
├── app.rs                  # State machine + event dispatch
├── error.rs                # Unified error types (thiserror)
├── config/                 # Configuration layer
│   ├── settings/           # TOML config + env overrides
│   ├── keymap/             # Vim/Emacs key binding config
│   └── watcher.rs          # Hot-reload (mtime + SIGHUP)
├── tui/                    # TUI rendering layer
│   ├── theme.rs            # Tokyo Night light/dark themes
│   ├── frame.rs            # 5-zone layout renderer
│   ├── diff_renderer.rs    # syntect diff highlighting
│   ├── syntax.rs           # Syntax highlighting engine
│   └── widgets/            # 7 composable UI components
├── llm/                    # LLM adapter layer
│   ├── adapter.rs          # LLM adapter trait
│   ├── stream.rs           # SSE stream parser
│   └── types.rs            # Unified LLM types
├── agent/                  # Agent layer
│   ├── agloop.rs           # Core agent loop
│   ├── context.rs          # Token budgeting + dynamic trimming
│   ├── prompt.rs           # System prompt builder
│   └── multi/              # Multi-agent collaboration
├── tools/                  # Tool layer
│   ├── registry.rs         # Tool registry
│   └── builtin/            # 5 built-in tools
├── skills/                 # Skills layer
│   ├── registry.rs         # Skill registry
│   ├── loader.rs           # YAML frontmatter parser
│   └── builtin/            # Built-in skills
├── mcp/                    # MCP protocol layer
│   ├── client.rs           # MCP client
│   ├── manager.rs          # Multi-server manager
│   └── transport.rs        # Transport trait
├── sandbox/                # Security sandbox
│   ├── command_preview.rs  # Risk classification
│   ├── atomic_replace.rs   # Crash-safe writes
│   └── approval.rs         # Approval workflows
├── workflow/               # Workflow engine
│   ├── definition.rs       # YAML workflow definition
│   ├── runner/             # Executor
│   └── builtin.rs          # Built-in workflows
└── session/                # Session persistence
    ├── store.rs            # SQLite persistence
    └── model.rs            # Session/message models
```

---

## 🧪 Testing

```bash
# Run all 1149 tests
cargo test

# Run specific suites
cargo test --lib                    # Unit tests
cargo test --test e2e               # End-to-end tests
cargo test --test phase2            # Tools + sandbox
cargo test --test phase3            # Theme + diff + syntax
cargo test --test phase4            # Keymaps + mouse
cargo test --test full_pipeline     # Full pipeline tests
cargo test --test workflow_integration  # Workflow integration

# Startup benchmark
cargo run --features startup_bench
```

---

## 🛠️ Development

```bash
# Type checking
cargo check

# Formatting
cargo fmt

# Static analysis
cargo clippy -- -D warnings -A dead_code

# Release build (LTO + strip)
cargo build --release
```

---

## 📊 Tech Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | 1.75+ |
| TUI Framework | ratatui + crossterm | 0.28 |
| Async Runtime | tokio | 1.x |
| HTTP Client | reqwest | 0.12 |
| Database | rusqlite (bundled) | 0.31 |
| Syntax Highlighting | syntect | 5.x |
| Error Handling | thiserror | 2.x |
| Configuration | toml | 0.8 |
| Logging | tracing | 0.1 |

---

## 📈 Performance

| Metric | Target | Actual | Status |
|--------|:------:|:------:|:------:|
| Startup | <80ms | **2ms** | ✅ |
| Binary Size | ≤15MB | **3.8MB** | ✅ |
| Tests | — | **1,149** | ✅ |
| Production `unwrap` | 0 | **0** | ✅ |
| `unsafe` Code | 0 | **0** | ✅ |
| Compiler Warnings | 0 | **0** | ✅ |

---

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

| Prefix | Scope | Example |
|--------|-------|---------|
| `feat:` | New feature | `feat: add streaming diff view` |
| `fix:` | Bug fix | `fix: resolve macos uuid race` |
| `docs:` | Documentation | `docs: update provider config` |
| `refactor:` | Refactoring | `refactor: extract agent loop` |
| `test:` | Tests | `test: add workflow coverage` |
| `chore:` | Build/tooling | `chore: update deps` |

---

## 📄 License

This project is [MIT Licensed](LICENSE) — free for personal and commercial use.

---

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo.svg">
    <img src="assets/logo.svg" alt="Synerix" width="96">
  </picture>
  <br><br>
  <strong>Synerix</strong> — AI-Native Coding Terminal<br>
  <sub>Built with Rust · Designed for AI · Driven by Performance</sub>
  <br><br>
  <a href="https://github.com/Agions/synerix"><img src="https://img.shields.io/badge/GitHub-Agions%2Fsynerix-181717.svg?style=flat-square&logo=github" alt="GitHub"></a>
  <a href="https://gitee.com/Agions/synerix"><img src="https://img.shields.io/badge/Gitee-Agions%2Fsynerix-C71D23.svg?style=flat-square&logo=gitee" alt="Gitee"></a>
</p>
