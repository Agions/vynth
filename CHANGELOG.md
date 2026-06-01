# Changelog

All notable changes to Synerix will be documented in this file.

## [0.1.1] — 2026-06-01

### 🎛️ Slash Command System Redesign

- **Registry architecture**: Replaced hardcoded `match` routing with declarative `CmdDef` registration table — commands are now self-documenting structs with name, description, category, aliases, and handler
- **Unified argument parsing**: `subcmd()`, `nth_arg()`, `rest_from()` helper functions replace ad-hoc per-command parsing logic
- **Hierarchical help**: `/help` now displays commands grouped by category (💡 Help, 📋 Session, 🤖 Model, ⚙️ Config, 🎯 Goal, 📦 Workflow); `/help <cmd>` shows aliases and usage
- **Alias support**: `/h`, `/?`, `/c`, `/cls`, `/m`, `/re`, `/quit`, `/q`, `/wf`, `/skills`, `/cfg`, `/g` shortcuts for common commands
- **15 new tests** covering alias resolution, help system hierarchy, and command completeness

### 🧹 Audit Fixes

- **Release profile**: `lto = true` → `lto = "fat"` for cross-crate LTO optimization
- **CI**: Added `cargo audit` step for vulnerability scanning
- **Features**: Restructured with `default = ["tui"]`, added `headless` mode feature
- **TokenBudget**: Hardcoded `2000/3000/4096` replaced with named associated constants (`DEFAULT_SYSTEM_OVERHEAD`, `DEFAULT_TOOLS_OVERHEAD`, `DEFAULT_RESERVED`)
- **Security**: `Sandbox::Auto` mode now has explicit `⚠️ Security Warning` doc annotation
- **Documentation**: Added `//!` module-level docs to 5 command/workflow files

## [0.1.0] — 2026-06-01

### 🏗️ Architecture

- **Cargo Workspace**: Monolithic crate split into workspace with `synerix` (main) and `synerix-core` (core abstractions) sub-crates
- **Core crate (`synerix-core`)**: Extracted shared types including unified `Role` enum, `MutexExt` trait, and datetime utilities
- **Lint cleanup**: Removed `#![allow(dead_code, unused_imports, unused_variables)]` from lib root; all warnings now treated as errors via clippy CI
- **CI pipeline**: Release CI with matrix build (linux/macos/windows), uploaded artifacts, auto-generated release notes
- **Binary size**: 3.9MB release build with `lto=true`, `panic="abort"`, `strip=true`

### ✂️ Module Splitting

- **`tui/frame.rs`** → `layout.rs` (pure layout computation) + `renderer.rs` (orchestration) + 5 dedicated widget files
- **`agent/agloop.rs`** → `agent_loop.rs` (core loop) + `tool_dispatcher.rs` (timeout-aware tool dispatch)
- **`app/`** → reorganized into `state.rs`, `events.rs`, `message.rs`, `input_handler.rs`, `runner.rs`, `actions.rs`

### 📛 Naming Conventions

- Unified all `.rs` filenames to `snake_case` (7 renames: `trait_def` → `traits`, `watcher` → `config_watcher`, `command_preview` → `risk_classifier`, `atomic_replace` → `atomic_writer`, `loader` → `skill_loader`, `client` → `mcp_client`)
- CI now enforces naming conventions in PR checks

### 🎨 Code Quality

- **TUI style constants**: Added 10 color aliases and 7 helper functions in `theme.rs`; replaced ~45 inline style constructions across all widgets
- **Token budget**: Introduced `TokenBudget::from_config()` to eliminate hardcoded magic numbers (2000/3000); added `system_prompt_tokens` and `tools_schema_tokens` to `LlmConfig`
- **Mutex handling**: Unified via `MutexExt::lock_or_err()` trait

### 🧪 Testing

- Added 4 new unit tests for datetime parsing utility
- **1275 tests total**, all passing with zero clippy warnings

### 🔧 Configuration

- `panic = "abort"` in release profile for smaller binaries
- Features renamed to kebab-case (`startup-bench`)
- `tokio` features narrowed from `full` to precise list

### 🚀 Release

- Added CI Release pipeline (`release.yml`) for automated GitHub Releases on tags
- Binary size: **3.9 MB** (under 5 MB target)
- Zero `unsafe` blocks, zero nightly features

### ⚡ Performance

- **SessionStore locking**: Consolidate to `Mutex<Connection>` with WAL mode for minimal contention (`rusqlite::Connection` is `Send` but not `Sync`)
- **TUI dirty-flag rendering**: Per-widget dirty flags skip unchanged widgets (sidebar, chat, diff, input, status) — 60fps CPU reduction
- **Configurable tool timeout**: `SandboxConfig.tool_timeout_secs` (default 120s) controls tool execution timeout, configurable via `config.toml`
- **Benchmark suite**: `criterion` benchmarks for token estimation, context push/trim, session CRUD — run with `cargo bench`
