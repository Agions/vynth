# Changelog

All notable changes to Syncode will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.1] - 2026-05-30

### Added
- npm installation method (`npm install -g syncode` / `npx syncode`)
- npm package with auto-download postinstall script (Linux/macOS, x86_64/arm64)

### Removed
- Homebrew installation method (removed `homebrew/syncode.rb`)

## [1.0.0] - 2026-05-30

### Added
- End-to-end integration tests with mock LLM server
- CI/CD pipeline (Gitee Actions: check, fmt, clippy, release build)
- Issue/PR templates
- Comprehensive CHANGELOG
- Production-ready packaging

### Changed
- Version bumped to 1.0.0

## [0.3.0] - 2026-05-30

### Added
- Vim keybinding profile (modal: Normal/Insert/Command/Search)
  - Normal: i/a/A/o/O→Insert, dd→clear, yy→yank, p→paste, Ctrl+d/u→page
  - Insert: Ctrl+w→delete word, Ctrl+k/u→kill to end/start
- Emacs keybinding profile (non-modal)
  - Ctrl+n/p→scroll, Ctrl+f/b→cursor, Ctrl+k→kill, Ctrl+y→yank
- Action enum with 25+ variants, KeyBindings resolve system
- Mouse support: click-to-focus, scroll wheel, sidebar tab switch
- FocusedPanel tracking with focus-aware border styling
- LayoutState for hit-testing panel rects
- StartupTimer + StartupMetrics instrumentation
- startup_bench feature flag for stderr timing output
- Config hot-reload: mtime polling (2s) + SIGHUP handler
- apply_config_reload() for non-restart config updates
- 106 tests total

## [0.2.0] - 2026-05-30

### Added
- Agent Loop: streaming inference → tool dispatch → multi-turn reasoning
- LLM Adapter: OpenAI-compatible API with SSE streaming
- 5 built-in tools: file_read, file_write, shell_exec, search, patch
- Sandbox: command risk classification (Safe→Critical), atomic writes, approval flow
- Context Manager: token budget, auto-trimming, message management
- Theme system: Tokyo Night dark + light with 20+ semantic colors
- Diff renderer: syntect syntax highlighting, unified + side-by-side views
- Syntax highlighting engine with cached SyntaxSet/ThemeSet
- TUI frame rendering: sidebar tabs, tool call display, scroll offset
- Real text input editing: cursor, insert, delete, Home/End, UTF-8
- 67 tests total

## [0.1.0] - 2026-05-30

### Added
- Project skeleton: 53 Rust source files, 3,664 lines
- ratatui + crossterm TUI event loop with 5-zone layout
- LLM adapter trait with OpenAI-compatible SSE parser
- Agent loop skeleton
- Tool trait + registry + 5 built-in tool skeletons
- Skill trait + registry + YAML loader + 2 built-in skills
- MCP client (stdio) + manager with permission isolation
- Sandbox: command preview + atomic write + approval skeletons
- SQLite session store (WAL mode)
- Config: TOML + environment variable overrides
- Unified AppError (thiserror)
- 22 tests total
