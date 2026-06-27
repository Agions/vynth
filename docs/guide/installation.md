# Installation

Complete installation guide for Synerix across all supported platforms.

## Requirements

### Minimum

- **OS**: Linux, macOS 12+, or Windows 10+
- **RAM**: 512 MB
- **Disk**: 50 MB free space
- **Network**: Internet access (for AI features)

### Recommended

- **OS**: Ubuntu 22.04+, macOS 13+, or Windows 11
- **RAM**: 2 GB
- **Disk**: 100 MB free space
- **Terminal**: Modern terminal with 256-color support (iTerm2, Windows Terminal, Alacritty, etc.)

## Install Methods

### 1. Install Script (Recommended)

Fastest way to get started with automatic PATH setup.

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

**Windows (PowerShell 5.1+):**

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### 2. Package Managers

**Homebrew (macOS / Linux):**

```bash
brew install agions/tap/synerix
```

**Cargo (Rust toolchain required):**

```bash
cargo install synerix
```

### 3. Build from Source

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
cargo build --release
```

The binary will be at `target/release/synerix`.

## Verify Installation

```bash
synerix --version
# synerix 0.2.2
```

## Uninstall

### Using Scripts

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash

# Linux / macOS — full wipe including config
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash -s -- --all

# Windows
# Use Add/Remove Programs or delete %LOCALAPPDATA%\Programs\synerix
```

### Manual

```bash
# Remove binary
rm ~/.local/bin/synerix

# Remove config (optional)
rm -rf ~/.config/synerix
```

## Post-Installation

1. Configure your LLM provider — see [Configuration](/guide/configuration)
2. Launch with `synerix`
3. Try your first AI-powered task

## Next Steps

- [Configuration](/guide/configuration) — Set up your AI provider and preferences
- [Coding Modes](/guide/modes) — Explore Act, Vibe, Chat, Architect, and Plan modes
- [Troubleshooting](/guide/troubleshooting) — Having issues? Find solutions here
