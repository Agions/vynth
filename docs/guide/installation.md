# 安装

本指南提供 Synerix 的详细安装说明。

## 系统要求

### 最低要求

- **操作系统**: Linux, macOS, Windows 10+
- **内存**: 512 MB RAM
- **磁盘空间**: 50 MB 可用空间
- **网络**: 需要互联网连接（用于 AI 功能）

### 推荐配置

- **操作系统**: Ubuntu 20.04+, macOS 12+, Windows 11
- **内存**: 2 GB RAM
- **磁盘空间**: 100 MB 可用空间
- **终端**: 支持 256 色的现代终端

## 安装方法

### 方法 1: 使用安装脚本（推荐）

#### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

#### Windows (PowerShell 5.1+)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSpread -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### 方法 2: 使用包管理器

#### Homebrew (macOS/Linux)

```bash
brew install agions/tap/synerix
```

#### Cargo (Rust)

```bash
cargo install synerix
```

### 方法 3: 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Agions/synerix.git
cd synerix

# 构建
cargo build --release

# 安装到系统路径
cargo install --path .
```

## 验证安装

```bash
# 检查版本
synerix --version

# 运行测试
synerix --test
```

## 卸载

### 使用安装脚本

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/uninstall.sh | bash

# Windows
irm https://raw.githubusercontent.com/Agions/synerix/main/uninstall.ps1 | iex
```

### 手动卸载

```bash
# 删除二进制文件
rm ~/.local/bin/synerix

# 删除配置文件
rm -rf ~/.config/synerix
```

## 下一步

- [配置](/guide/configuration) - 自定义你的 Synerix
- [使用模式](/guide/modes) - 了解不同的工作模式
