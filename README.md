# Syncode

> AI Pair Programming Terminal — Sync your code with AI

A high-performance, single-process TUI application that fuses Claude Code's interaction model, Codex CLI's sandbox mechanism, and OpenCode's extensible architecture.

## Features

- **TUI Interface** — ratatui-powered terminal UI with sidebar, chat, diff preview, status bar
- **LLM Integration** — DeepSeek V4 / MiMo-v2.5 / any OpenAI-compatible API
- **Agentic Loop** — Streaming inference → tool dispatch → continue reasoning
- **Tool System** — Pluggable tool registry with 5 built-in tools (file read/write, shell, search, patch)
- **Skills Tree** — YAML/MD skill files with auto-matching and system prompt injection
- **MCP Protocol** — Native MCP client with stdio/HTTP transport and permission isolation
- **Sandbox Security** — Command preview, atomic file writes, user approval flow
- **Session Persistence** — SQLite-based session and message storage

## Tech Stack

| Component | Choice |
|-----------|--------|
| Language | Rust (edition 2021) |
| TUI | ratatui 0.28 + crossterm 0.28 |
| Async | tokio 1.x (full features) |
| HTTP | reqwest 0.12 (json + stream) |
| SQLite | rusqlite 0.31 (bundled) |
| Error | thiserror 2.x |
| Config | toml 0.8 |
| Logging | tracing 0.1 |
| Syntax | syntect 5.x |

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
```

Requires a terminal emulator with truecolor support.

## Configuration

Config file: `~/.config/syncode/config.toml`

```toml
[llm]
provider = "deepseek"
api_key = "your-api-key"
model = "deepseek-chat"
context_window = 128000
max_output_tokens = 8192

[ui]
theme = "dark"
keymap = "default"

[sandbox]
mode = "confirm"
atomic_writes = true
```

Environment variables override config:
- `SYNCODE_API_KEY` — LLM API key
- `SYNCODE_BASE_URL` — API base URL
- `SYNCODE_MODEL` — Model identifier

## Architecture

```
src/
├── main.rs           # Entry point
├── app.rs            # App state machine + event loop
├── error.rs          # Unified error types
├── config/           # Settings + keymaps
├── tui/              # TUI rendering layer
│   └── widgets/      # Composable UI components
├── llm/              # LLM adapter layer
├── agent/            # Agentic loop + context manager
├── tools/            # Tool registry + built-in tools
├── skills/           # Skill registry + loader
├── mcp/              # MCP client + manager
├── sandbox/          # Security sandbox
└── session/          # SQLite session store
```

## License

MIT
