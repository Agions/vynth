# Changelog

All notable changes to Synerix are documented here.

## [0.2.2] — 2026-06-11

### Added

- Vibe mode auto-iteration: generate, compile, test, fix
- Smart sandbox with risk-aware approval flows
- Multi-provider LLM support (DeepSeek, custom OpenAI-compatible)
- Plugin system with command, skill, and MCP types
- Slash command framework with autocomplete
- Activity labels with animated status indicators
- Model catalog for discoverable presets

### Changed

- Token count removed from status bar for a cleaner UI
- Welcome screen replaced with ASCII art branding
- Activity labels now use accent color for active states
- TUI widgets refactored into composable components

### Fixed

- Terminal display corruption on resize
- Config hot reload reliability
- Memory leak in streaming pipeline

## [0.2.1] — 2026-05-15

### Added

- Chat mode with streaming responses
- Code review workflow
- Git integration for session management

### Changed

- Response latency reduced by 30%
- Error messaging improved across all modes

### Fixed

- AI connection retry logic
- Command history persistence

## [0.2.0] — 2026-04-01

### Added

- Act mode with sandboxed command execution
- Core AI integration layer
- TOML-based configuration system
- Tokyo Night theme with light/dark variants

## [0.1.0] — 2026-03-01

### Added

- Project initialization
- Core Rust workspace structure
- Basic terminal UI with ratatui

## Versioning

Synerix follows [Semantic Versioning](https://semver.org/):

- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible
