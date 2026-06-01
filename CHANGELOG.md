# Changelog

All notable changes to Synerix will be documented in this file.

## [0.1.0] — 2026-06-01

### 🏗️ Architecture

- **Cargo Workspace**: Monolithic crate split into workspace with `synerix` (main) and `synerix-core` (core abstractions) sub-crates
- **Core crate (`synerix-core`)**: Extracted shared types including unified `Role` enum, `MutexExt` trait, and datetime utilities
- **Lint cleanup**: Removed `#![allow(dead_code, unused_imports, unused_variables)]` from lib root; all warnings now treated as errors via clippy CI

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
