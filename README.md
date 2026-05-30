# Syncode

> AI Pair Programming Terminal — Sync your code with AI

[![CI](https://gitee.com/Agions/syncode/badges/master/pipeline.svg)](https://gitee.com/Agions/syncode/pipelines)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)

A high-performance, single-process TUI application that fuses Claude Code's interaction model, Codex CLI's sandbox mechanism, and OpenCode's extensible architecture.

## Features

| Feature | Description |
|---------|-------------|
| **Agentic Loop** | Streaming inference → tool dispatch → multi-turn reasoning |
| **LLM Integration** | DeepSeek V4, MiMo-v2.5, any OpenAI-compatible API |
| **5 Built-in Tools** | file_read/write, shell_exec, search (ripgrep), patch |
| **TUI Interface** | ratatui 5-zone layout with Tokyo Night theme |
| **Diff Highlighting** | syntect syntax highlighting, unified + side-by-side views |
| **Vim/Emacs Keybindings** | Full modal Vim + non-modal Emacs profiles |
| **Mouse Support** | Click-to-focus, scroll wheel, sidebar tab switch |
| **Skills System** | YAML/MD skill files with auto-matching |
| **MCP Protocol** | Native client with stdio/HTTP transport |
| **Sandbox Security** | Command risk classification, atomic writes, approval flow |
| **Config Hot-Reload** | mtime polling + SIGHUP signal |
| **Session Persistence** | SQLite (WAL mode) with full CRUD |

## Quick Start

```bash
# Install from source
cargo install --path .

# Or build locally
cargo build --release
./target/release/syncode
```

## Configuration

Config file: `~/.config/syncode/config.toml`

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

Environment variable overrides:
- `SYNCODE_API_KEY` — LLM API key
- `SYNCODE_BASE_URL` — API base URL
- `SYNCODE_MODEL` — Model identifier

## Keybindings

### Vim Profile

| Mode | Key | Action |
|------|-----|--------|
| Normal | `i` / `a` / `A` | Enter Insert mode |
| Normal | `:` / `/` | Command / Search mode |
| Normal | `j` / `k` | Scroll down / up |
| Normal | `G` | Scroll to bottom |
| Normal | `dd` | Clear line |
| Normal | `yy` / `p` | Yank / Paste |
| Insert | `Esc` | Back to Normal |
| Insert | `Ctrl+w` | Delete word |
| Insert | `Ctrl+k` / `Ctrl+u` | Kill to end / start |

### Emacs Profile

| Key | Action |
|-----|--------|
| `Ctrl+n` / `Ctrl+p` | Scroll down / up |
| `Ctrl+f` / `Ctrl+b` | Cursor right / left |
| `Ctrl+a` / `Ctrl+e` | Home / End |
| `Ctrl+k` / `Ctrl+y` | Kill / Yank |

## Architecture

```
src/
├── main.rs           # Entry point + startup instrumentation
├── app.rs            # App state machine + event dispatch
├── error.rs          # Unified error types (thiserror)
├── lib.rs            # Library crate re-exports
├── telemetry.rs      # Startup metrics
├── config/
│   ├── settings.rs   # TOML config + env overrides
│   ├── keymap.rs     # Vim/Emacs keybinding profiles
│   └── watcher.rs    # Config hot-reload (mtime + SIGHUP)
├── tui/
│   ├── theme.rs      # Tokyo Night dark/light themes
│   ├── frame.rs      # 5-zone layout rendering
│   ├── diff_renderer.rs  # syntect diff highlighting
│   ├── syntax.rs     # Code highlighting engine
│   ├── event.rs      # crossterm event source
│   └── widgets/      # 7 composable UI components
├── llm/
│   ├── adapter.rs    # LLM adapter trait + OpenAI compat
│   ├── stream.rs     # SSE stream parser
│   └── types.rs      # Unified LLM types
├── agent/
│   ├── agloop.rs     # Core agentic loop
│   ├── context.rs    # Token budget + dynamic trimming
│   └── prompt.rs     # System prompt builder
├── tools/
│   ├── registry.rs   # Tool registry
│   └── builtin/      # 5 built-in tools
├── skills/
│   ├── registry.rs   # Skill registry
│   ├── loader.rs     # YAML frontmatter parser
│   └── builtin/      # Built-in skills
├── mcp/
│   ├── client.rs     # MCP client (stdio)
│   ├── manager.rs    # Multi-server manager
│   └── transport.rs  # Transport trait
├── sandbox/
│   ├── command_preview.rs  # Risk classification
│   ├── atomic_replace.rs   # Crash-safe writes
│   └── approval.rs         # Approval flow
└── session/
    ├── store.rs      # SQLite persistence
    └── model.rs      # Session/Message models
```

## Tech Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | 1.75+ |
| TUI | ratatui + crossterm | 0.28 |
| Async | tokio | 1.x |
| HTTP | reqwest | 0.12 |
| SQLite | rusqlite (bundled) | 0.31 |
| Syntax | syntect | 5.x |
| Error | thiserror | 2.x |
| Config | toml | 0.8 |
| Logging | tracing | 0.1 |

## Testing

```bash
# Run all tests (115 tests)
cargo test

# Run specific test suite
cargo test --test e2e          # End-to-end with mock LLM
cargo test --test phase2       # Tools + sandbox
cargo test --test phase3       # Theme + diff + syntax
cargo test --test phase4       # Keybindings + mouse

# Run with startup benchmark
cargo run --features startup_bench
```

## Development

```bash
# Check
cargo check

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings -A dead_code

# Build release
cargo build --release
```

## License

MIT
