# Getting Started / 快速开始

Welcome to **Synerix** — an AI-native coding terminal designed to keep you in flow.

欢迎使用 **Synerix** —— 一款专为保持心流而设计的 AI 原生编程终端。

## What is Synerix? / Synerix 是什么？

Synerix is a high-performance terminal that combines the power of modern AI with the speed of a native CLI. It thinks, writes, reviews, and fixes code — without pulling you out of the command line.

Synerix 是一款高性能终端，将现代 AI 的能力与原生 CLI 的速度相结合。它会思考、编写、审查和修复代码 —— 而不会让你离开命令行。

## Quick Install / 快速安装

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

### Windows (PowerShell 5.1+)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### From Source / 从源码构建

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
cargo build --release
cargo install --path .
```

## First Launch / 首次启动

After installation, configure your LLM provider:

安装后，配置你的 LLM 提供商：

```bash
# Create config directory / 创建配置目录
mkdir -p ~/.config/synerix

# Create config file / 创建配置文件
cat > ~/.config/synerix/config.toml << 'EOF'
[llm]
provider = "deepseek"
api_key = "sk-..."
model = "deepseek-v4-flash"
EOF
```

Then launch:

然后启动：

```bash
synerix
```

## Your First Session / 你的第一个会话

```
❯ Refactor main.rs to reduce nesting
   ✓ Code generated
   ✓ cargo check passes
   ✓ Tests pass
   ✓ Done in 1.8s

❯ /mode vibe
   Switched to Vibe mode — auto-iterate enabled

❯ Add pagination to the API endpoint
   ✓ Generated pagination module
   ✓ Compiled successfully
   ✓ All tests green
```

## Common Commands / 常用命令

| Command / 命令 | Description / 描述 |
|---|---|
| `synerix` | Launch the interactive terminal / 启动交互式终端 |
| `synerix --help` | Show help information / 显示帮助信息 |
| `synerix --version` | Show version / 显示版本 |
| `/mode <name>` | Switch coding mode (act, vibe, chat, architect, plan) / 切换编程模式 |
| `/help` | List all slash commands / 列出所有斜杠命令 |
| `/clear` | Clear conversation / 清空对话 |
| `/exit` | Quit Synerix / 退出 Synerix |

## Keybindings / 快捷键

| Key / 按键 | Action / 操作 |
|---|---|
| `Tab` | Auto-complete / switch mode / 自动补全 / 切换模式 |
| `Ctrl+C` | Cancel current operation / 取消当前操作 |
| `Ctrl+D` | Exit / 退出 |
| `Ctrl+L` | Clear screen / 清屏 |
| `↑ / ↓` | Navigate history / 浏览历史 |

## Next Steps / 下一步

- [Installation](/guide/installation) — Detailed installation guide / 详细安装指南
- [Configuration](/guide/configuration) — Customize Synerix to your workflow / 根据你的工作流定制 Synerix
- [Coding Modes](/guide/modes) — Learn about Plan and Vibe modes / 了解规划模式和沉浸模式
- [Troubleshooting](/guide/troubleshooting) — Common issues and solutions / 常见问题与解决方案
