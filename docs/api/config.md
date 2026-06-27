# Configuration API

Programmatic configuration reference for plugins, agents, and integrations.

## File Locations

| Platform | Path |
|---|---|
| Linux / macOS | `~/.config/synerix/config.toml` |
| Windows | `%APPDATA%\synerix\config.toml` |

## Full Configuration Schema

```toml
[llm]
provider = "deepseek"          # deepseek, custom
model = "deepseek-v4-flash"
api_key = "sk-..."
base_url = "https://api.deepseek.com"

[ui]
theme = "dark"                 # dark, light
keymap = "default"             # default, vim, emacs
animation = true

[sandbox]
mode = "confirm"               # auto, confirm, strict

[agents.architect]
system_prompt = "..."
tools = ["file_read", "search"]
max_turns = 8

[[mcp]]
name = "filesystem"
type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]

[skills]
sources = ["builtin", "~/.synerix/skills"]

[workflows.pipelines.code_review]
steps = ["architect", "coder", "reviewer", "tester"]
```

## Runtime Config Access

```rust
use synerix::config::Config;

let config = Config::load()?;
let model = config.llm.model;
let provider = config.llm.provider;
```

## Environment Variables

| Variable | Description |
|---|---|
| `SYNERIX_API_KEY` | LLM API key |
| `SYNERIX_BASE_URL` | Custom API base URL |
| `SYNERIX_MODEL` | Override default model |
| `SYNERIX_PROVIDER` | Override default provider |

## Validation

Synerix validates configuration on load:

```rust
use synerix::config::validation;

validation::validate(&config)?;
```

Invalid configs produce clear error messages pointing to the problematic field.

## Next Steps

- [Plugins](/api/plugins) — Build plugins that read and modify config
- [Troubleshooting](/guide/troubleshooting) — Config issues
