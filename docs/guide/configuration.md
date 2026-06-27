# Configuration

Complete reference for configuring Synerix to match your workflow.

## Configuration File

Location: `~/.config/synerix/config.toml`

## Basic Setup

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

## LLM Providers

### DeepSeek (Default)

```toml
[llm]
provider = "deepseek"
model = "deepseek-v4-flash"
api_key = "sk-..."
```

### Custom OpenAI-Compatible

```toml
[llm]
provider = "custom"
model = "your-model"
base_url = "https://api.example.com/v1"
api_key = "sk-..."
```

### Environment Variables

You can override any config value with environment variables:

| Variable | Description |
|---|---|
| `SYNERIX_API_KEY` | LLM API key |
| `SYNERIX_BASE_URL` | Custom API endpoint |
| `SYNERIX_MODEL` | Model override |

## UI Settings

```toml
[ui]
theme = "dark"        # dark, light
keymap = "default"    # default, vim, emacs
animation = true
```

## Sandbox Modes

```toml
[sandbox]
mode = "confirm"      # auto, confirm, strict
```

| Mode | Behavior |
|---|---|
| `auto` | Safe operations run instantly |
| `confirm` | Risky operations require preview |
| `strict` | All operations require confirmation |

## Slash Commands Reference

| Command | Description |
|---|---|
| `/mode <name>` | Switch mode: `act`, `vibe`, `chat`, `architect`, `plan` |
| `/help` | Show available commands |
| `/clear` | Clear conversation |
| `/model <name>` | Switch model preset |
| `/exit` | Exit Synerix |

## MCP Servers

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

## Custom Agents

```toml
[[agents]]
name = "security-auditor"
system_prompt = "You are a security expert. Find vulnerabilities."
tools = ["file_read", "search"]
max_turns = 8
```

## Hot Reload

Synerix watches `config.toml` for changes and reloads automatically. You can also trigger a reload with:

```bash
kill -HUP $(pgrep synerix)
```

## Next Steps

- [Coding Modes](/guide/modes) — Learn about different work modes
- [Troubleshooting](/guide/troubleshooting) — Common issues
