# 快速开始

欢迎使用 Synerix！这是一个 AI 原生的编码终端，帮助你在终端中思考、编写、审查和修复代码。

## 安装

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

### Windows (PowerShell 5.1+)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSpread -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Agions/synerix.git
cd synerix

# 构建
cargo build --release

# 安装
cargo install --path .
```

## 基本使用

### 启动 Synerix

```bash
synerix
```

### 常用命令

| 命令 | 说明 |
|------|------|
| `synerix` | 启动交互式终端 |
| `synerix --help` | 显示帮助信息 |
| `synerix --version` | 显示版本号 |
| `synerix config` | 配置设置 |

### 快捷键

| 快捷键 | 说明 |
|--------|------|
| `Ctrl+C` | 中断当前操作 |
| `Ctrl+D` | 退出 |
| `Ctrl+L` | 清屏 |
| `Tab` | 自动补全 |

## 下一步

- [安装](/guide/installation) - 详细安装说明
- [配置](/guide/configuration) - 自定义你的 Synerix
- [使用模式](/guide/modes) - 了解不同的工作模式
- [故障排除](/guide/troubleshooting) - 常见问题解决方案
