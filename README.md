<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/banner.svg">
    <img src="assets/banner.svg" alt="Synerix — AI-Native Coding Terminal" width="100%">
  </picture>
</p>

<div align="center">

**Synerix** is an AI-native coding terminal that **thinks, writes, reviews, and fixes code** — without pulling you out of the command line.

[⚡ Act] · [🎵 Vibe] · [🗣 Chat] · [📐 Architect] · [📋 Plan]

</div>

<p align="center">
  <a href="https://github.com/Agions/synerix/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Agions/synerix/ci.yml?branch=main&style=flat-square&label=CI&logo=github" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-00D4FF.svg?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75%2B-FF6B3D.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust"></a>
  <img src="https://img.shields.io/badge/Tests-706-00FF88.svg?style=flat-square&logo=rust" alt="Tests">
  <img src="https://img.shields.io/badge/Binary-3.8_MB-7B61FF.svg?style=flat-square" alt="Size">
  <img src="https://img.shields.io/badge/Unsafe_Code-0-FF4444.svg?style=flat-square" alt="Unsafe">
  <img src="https://img.shields.io/badge/Mode_Vibe-✨-507882.svg?style=flat-square" alt="Vibe Mode">
</p>

---

## Why Synerix?

AI coding assistants are powerful — but the **workflow friction** kills your flow:

| The Problem | Synerix's Answer |
|---|---|
| Switch between editor, browser, and terminal 50× an hour | **One terminal.** Type, generate, review, run, fix — all in one place. |
| Approve every file write, every command, every keystroke | **Smart sandbox.** Low-risk ops auto-approve in Vibe mode; dangerous ops get a full preview. |
| Paste code into chat, copy it back, hit compile, paste errors back | **Auto-iteration.** `described → generated → compiled → tested → fixed` in a single loop. |

Synerix collapses the AI coding workflow from **6 tools × 3 contexts** into **1 terminal window**.

---

## ⚡ Installation

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

### Windows (PowerShell 5.1+)

```powershell
# Recommended — one-liner (equivalent to curl | bash)
iwr -useb https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

> **Alternative package managers** (once published):
>
> | Manager | Command | Notes |
> |---|---|---|
> | **winget** | `winget install Agions.synerix` | Built into Win10/11 |
> | **scoop** | `scoop bucket add Agions; scoop install synerix` | Dev favourite |
> | **choco** | `choco install synerix` | Enterprise |
> | **cargo** | `cargo install synerix` | Rust toolchain required |

<details>
<summary><b>China mirror (Gitee)</b></summary>

```bash
# Linux / macOS
curl -fsSL https://gitee.com/Agions/synerix/raw/main/install.sh | bash
```

```powershell
# Windows
iwr -useb https://gitee.com/Agions/synerix/raw/main/install.ps1 | iex
```
</details>

<details>
<summary><b>Build from source</b></summary>

```bash
git clone https://github.com/Agions/synerix.git
cd synerix && cargo build --release
# Binary → target/release/synerix (or synerix.exe on Windows)
```
</details>

<details>
<summary><b>Uninstall</b></summary>

```bash
# Linux / macOS — Keep config
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash

# Linux / macOS — Wipe everything
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash -s -- --all

# Windows — Remove binary
Remove-Item "$env:LOCALAPPDATA\Programs\synerix" -Recurse -Force
```
</details>

---

## ✨ Capabilities

<table>
<tr>
<td width="50%">

### 🤖 Multi-Agent Swarm
5 specialized agents work together like a dev team:

| Role | Does |
|---|---|
| **Coder** | Writes and modifies code |
| **Reviewer** | Reviews for quality |
| **Tester** | Creates and runs tests |
| **Architect** | Designs the structure |
| **Planner** | Breaks down tasks |

YAML-defined pipelines chain them: `code-review (Coder→Reviewer→Tester)`, `refactor (Architect→Coder→Reviewer)`, `debug (Tester→Coder→Tester)`.

</td>
<td width="50%">

### 🎵 Vibe Coding Mode
**Describe → Generate → Compile → Fix.** Zero friction.

```
❯ Add user preferences to the API
  ✓ Code generated
  ✓ `cargo check` passes
  ✓ Tests pass
  ✓ Done. 1.8s elapsed.
```

`/mode vibe` switches to immersive mode: auto-approve low-risk ops, auto-fix compile errors, auto-verify every change. No prompts, no context switches — just flow.

`/mode v` · `/mode 沉浸` · `/mode act` to exit.

</td>
</tr>
<tr>
<td width="50%">

### 🎨 Terminal UI
ratatui 5-zone layout with Tokyo Night palette:

- **Rounded borders** on every panel
- **Role-colored chat:** User `▸` · AI `◆` · Tool `⚙`
- **Syntax-highlighted diff** (syntect)
- **5 mode colors:** Plan · Act · Chat · Architect · **Vibe**
- **Vim / Emacs / Default** keymaps
- Dark & Light themes

</td>
<td width="50%">

### 🔒 Security Sandbox
3 protection layers, no false alarms:

| Level | Mode | Behavior |
|---|---|---|
| ⬜ `auto` | Safe ops run | instantly |
| 🟨 `confirm` | Risky ops | previewed |
| 🟥 `preview` | Dangerous ops | blocked |

- **Atomic writes** — crash-safe file updates
- **Risk classifier** — `ls` ≠ `rm -rf` ≠ `sudo`
- **MCP isolation** — per-tool permission boundaries

</td>
</tr>
<tr>
<td width="50%">

### 🧩 Skill System
Auto-detected YAML/MD skill files + external sources:

```
.synerix/
├── skills/          # Project-local skills
│   └── code-review.md
└── agents/          # Custom agent definitions
    └── security-auditor.yaml
```

Load from Git repos, local directories, or built-in defaults. `/skill` to manage.

</td>
<td width="50%">

### ⚡ Blazing Performance
Written in Rust, built for speed:

| Metric | Result |
|---|---|
| Startup | **2ms** |
| Binary | **3.8 MB** |
| Unsafe code | **0 lines** |
| Compiler warnings | **0** |
| Dependencies | Minimal, zero-copy path |

LTO + strip + `codegen-units=1`. Token budgeting + LRU cache throughout.

</td>
</tr>
</table>

---

## 🚀 Launch & First Run

```bash
# 1. Configure your LLM
mkdir -p ~/.config/synerix

cat > ~/.config/synerix/config.toml << 'EOF'
[llm]
provider = "deepseek"
api_key = "sk-..."
model = "deepseek-v4-flash"

[ui]
theme = "dark"
keymap = "default"

[sandbox]
mode = "confirm"
EOF

# 2. Launch
synerix

# 3. Try it
❯ Refactor main.rs to reduce nesting
❯ /mode vibe                      # Enter flow state
❯ Add pagination to the API       # Auto-generate, compile, test, fix
```

---

## 🔌 Configuration

### LLM Providers

| Provider | Default Model | Notes |
|---|---|---|
| `deepseek` | deepseek-v4-flash | Default |
| `mimo` | mimo-v2.5-pro | Xiaomi MiMo |
| `custom` | any | OpenAI-compatible API |

Environment overrides: `SYNERIX_API_KEY`, `SYNERIX_BASE_URL`, `SYNERIX_MODEL`.

### MCP Servers

```toml
# Remote (HTTP)
[mcp]
name = "remote-tools"
type = "http"
url = "https://mcp.example.com/sse"

# Local (stdio)
[mcp]
name = "filesystem"
type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
```

### Custom Agents

```toml
[[agents]]
name = "security-auditor"
system_prompt = "You are a security expert. Find vulnerabilities."
tools = ["file_read", "search"]
max_turns = 8
```

### Slash Commands

| Command | What it does |
|---|---|
| `/mode <mode>` | Switch coding mode (act / vibe / chat / architect / plan) |
| `/help` | List all commands |
| `/clear` | Clear conversation |
| `/model <name>` | Switch LLM model |
| `/reset` | Reset session state |
| `/exit` | Quit Synerix |
| `/mcp` / `/mcp list` | List MCP servers |
| `/mcp add <name> stdio <cmd>` | Add stdio MCP server |
| `/mcp add <name> http <url>` | Add HTTP MCP server |
| `/mcp remove <name>` | Remove MCP server |
| `/skill` | Manage skills and sources |
| `/workflow <name>` | Run a workflow |

---

## 🏗️ Architecture

```
src/
├── main.rs                 # Entry point + startup timer
├── app.rs                  # State machine + event dispatch
├── error.rs                # Unified errors (thiserror)
├── config/                 # Configuration layer
│   ├── settings/           # TOML config + env overrides
│   ├── keymap/             # Vim/Emacs/default keymaps
│   └── watcher.rs          # Hot-reload (mtime + SIGHUP)
├── tui/                    # Terminal UI
│   ├── theme.rs            # Tokyo Night → ColorPalette
│   ├── layout.rs           # 5-zone layout computation
│   ├── renderer.rs         # Dirty-flag diff renderer
│   └── widgets/            # 7 composable UI widgets
├── agent/                  # Agent core
│   ├── loop.rs             # Agent loop (tool dispatch + LLM stream)
│   ├── context.rs          # Token budgeting + dynamic trim
│   ├── prompt.rs           # System prompt builder
│   ├── modes.rs            # 5 coding modes (+ Vibe)
│   └── multi/              # Multi-agent swarm + bus
├── tools/                  # Tool layer
│   ├── registry.rs         # Schema-caching registry
│   └── builtin/            # file_read, file_write, patch, search, shell_exec
├── skills/                 # Skills system
├── mcp/                    # MCP protocol (stdio + HTTP)
├── sandbox/                # Security sandbox
│   ├── approval.rs         # Approval workflows
│   ├── classifier.rs       # Risk classification
│   └── atomic.rs           # Crash-safe file writes
├── workflow/               # DAG workflow engine
├── session/                # SQLite session persistence
├── coding_modes.rs         # Coding mode definitions
└── telemetry.rs            # Startup metrics + timing
```

---

## 🧪 Testing & Development

```bash
cargo check              # Type-check (fast)
cargo test --workspace   # 706 tests
cargo clippy             # Zero warnings
cargo build --release    # LTO + strip → 3.8 MB
```

| Test Suite | Count | What it covers |
|---|---|---|
| `--lib` | 505 unit tests | Agents · config · commands · MCP · sandbox · skills · tools · TUI · workflows |
| `--test e2e` | 9 tests | File tools · keymaps · LLM mock · session persistence |
| `--test full_pipeline` | 41 tests | Config→tools→context pipeline, session lifecycle, skill matching |
| `--test phase2` | 25 tests | Atomic writes · command preview · risk classification |
| `--test phase3` | 20 tests | Theme system · syntax highlight · diff rendering |
| `integration` | 16 tests | App creation · settings roundtrip · JSON-RPC · token budgets |
| `workflow_integration` | 26 tests | DAGs · conditions · retry · timeout · variable propagation |
| `sandbox_pipeline` | 14 tests | Atomic write safety · injection detection · permission isolation |

---

## 📄 License

MIT — free for personal and commercial use.

---

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo.svg">
    <img src="assets/logo.svg" alt="Synerix" width="80">
  </picture>
  <br><br>
  <a href="https://github.com/Agions/synerix"><img src="https://img.shields.io/badge/GitHub-Agions%2Fsynerix-181717.svg?style=flat-square&logo=github" alt="GitHub"></a>
  <a href="https://gitee.com/Agions/synerix"><img src="https://img.shields.io/badge/Gitee-Agions%2Fsynerix-C71D23.svg?style=flat-square&logo=gitee" alt="Gitee"></a>
  <a href="#"><img src="https://img.shields.io/badge/Written_in-Rust-00D4FF.svg?style=flat-square&logo=rust" alt="Rust"></a>
</p>