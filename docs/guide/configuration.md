# Configuration / 配置指南

Complete reference for configuring Synerix to match your workflow.

Synerix 完整配置参考，可根据你的工作流进行调整。

## Configuration File / 配置文件

Location: `~/.config/synerix/config.toml`

位置：`~/.config/synerix/config.toml`

## Basic Setup / 基本设置

```toml
[llm]
provider = "deepseek"
api_key = "sk-..."
model = "deepseek-v4-flash"

[ui]
theme = "dark"
keymap = "default"

[sandbox]
mode = "confirm"
```

## LLM Providers / LLM 提供商

### DeepSeek (Default) / DeepSeek（默认）

```toml
[llm]
provider = "deepseek"
model = "deepseek-v4-flash"
api_key = "sk-..."
```

### Custom OpenAI-Compatible / 自定义 OpenAI 兼容

```toml
[llm]
provider = "custom"
model = "your-model"
base_url = "https://api.example.com/v1"
api_key = "sk-..."
```

### Environment Variables / 环境变量

You can override any config value with environment variables:

你可以使用环境变量覆盖任何配置值：

| Variable / 变量 | Description / 描述 |
|---|---|
| `SYNERIX_API_KEY` | LLM API key / LLM API 密钥 |
| `SYNERIX_BASE_URL` | Custom API endpoint / 自定义 API 端点 |
| `SYNERIX_MODEL` | Model override / 模型覆盖 |

## UI Settings / 界面设置

```toml
[ui]
theme = "dark"        # dark, light / 深色、浅色
keymap = "default"    # default, vim, emacs
animation = true
```

## Sandbox Modes / 沙箱模式

```toml
[sandbox]
mode = "confirm"      # auto, confirm, strict / 自动、确认、严格
```

| Mode / 模式 | Behavior / 行为 |
|---|---|
| `auto` | Safe operations run instantly / 安全操作立即执行 |
| `confirm` | Risky operations require preview / 风险操作需要预览 |
| `strict` | All operations require confirmation / 所有操作都需要确认 |

## Slash Commands Reference / 斜杠命令参考

| Command / 命令 | Description / 描述 |
|---|---|
| `/mode <name>` | Switch mode: `act`, `vibe`, `chat`, `architect`, `plan` / 切换模式 |
| `/help` | Show available commands / 显示可用命令 |
| `/clear` | Clear conversation / 清空对话 |
| `/model <name>` | Switch model preset / 切换模型预设 |
| `/exit` | Exit Synerix / 退出 Synerix |

## MCP Servers / MCP 服务器

```toml
[[mcp]]
name = "filesystem"
type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

[[mcp]]
name = "remote-tools"
type = "http"
url = "https://mcp.example.com/sse"
```

## Custom Agents / 自定义代理

```toml
[[agents]]
name = "security-auditor"
system_prompt = "You are a security expert. Find vulnerabilities."
tools = ["file_read", "search"]
max_turns = 8
```

## Hot Reload / 热重载

Synerix watches `config.toml` for changes and reloads automatically. You can also trigger a reload with:

Synerix 会监听 `config.toml` 的更改并自动重载。你也可以通过以下方式触发重载：

```bash
kill -HUP $(pgrep synerix)
```

## Next Steps / 下一步

- [Coding Modes](/guide/modes) — Learn about different work modes / 了解不同的工作模式
- [Troubleshooting](/guide/troubleshooting) — Common issues / 常见问题
