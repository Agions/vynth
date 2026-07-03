# Installation / 安装指南

Complete installation guide for Synerix across all supported platforms.

Synerix 在所有支持平台上的完整安装指南。

## Requirements / 系统要求

### Minimum / 最低配置

- **OS**: Linux, macOS 12+, or Windows 10+ / 操作系统：Linux、macOS 12+ 或 Windows 10+
- **RAM**: 512 MB / 内存：512 MB
- **Disk**: 50 MB free space / 磁盘：50 MB 可用空间
- **Network**: Internet access (for AI features) / 网络：互联网连接（用于 AI 功能）

### Recommended / 推荐配置

- **OS**: Ubuntu 22.04+, macOS 13+, or Windows 11 / 操作系统：Ubuntu 22.04+、macOS 13+ 或 Windows 11
- **RAM**: 2 GB / 内存：2 GB
- **Disk**: 100 MB free space / 磁盘：100 MB 可用空间
- **Terminal**: Modern terminal with 256-color support (iTerm2, Windows Terminal, Alacritty, etc.) / 终端：支持 256 色的现代终端（iTerm2、Windows Terminal、Alacritty 等）

## Install Methods / 安装方法

### 1. Install Script (Recommended) / 安装脚本（推荐）

Fastest way to get started with automatic PATH setup.

最快上手方式，自动配置 PATH。

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

**Windows (PowerShell 5.1+):**

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### 2. Package Managers / 包管理器

**Homebrew (macOS / Linux):**

```bash
brew install agions/tap/synerix
```

**Cargo (Rust toolchain required):**

```bash
cargo install synerix
```

### 3. Build from Source / 从源码构建

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
cargo build --release
```

The binary will be at `target/release/synerix`.

二进制文件将位于 `target/release/synerix`。

## Verify Installation / 验证安装

```bash
synerix --version
# synerix 0.2.3
```

## Uninstall / 卸载

### Using Scripts / 使用脚本

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash

# Linux / macOS — full wipe including config / 完全清除包括配置
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash -s -- --all

# Windows
# Use Add/Remove Programs or delete %LOCALAPPDATA%\Programs\synerix
```

### Manual / 手动卸载

```bash
# Remove binary / 删除二进制文件
rm ~/.local/bin/synerix

# Remove config (optional) / 删除配置（可选）
rm -rf ~/.config/synerix
```

## Post-Installation / 安装后

1. Configure your LLM provider — see [Configuration](/guide/configuration) / 配置你的 LLM 提供商 —— 查看 [配置指南](/guide/configuration)
2. Launch with `synerix` / 使用 `synerix` 启动
3. Try your first AI-powered task / 尝试你的第一个 AI 任务

## Next Steps / 下一步

- [Configuration](/guide/configuration) — Set up your AI provider and preferences / 设置 AI 提供商和偏好
- [Coding Modes](/guide/modes) — Explore Plan and Vibe modes / 探索规划模式和沉浸模式
- [Troubleshooting](/guide/troubleshooting) — Having issues? Find solutions here / 遇到问题？在这里找到解决方案
