# Commands

Comprehensive reference for creating and registering commands in Synerix.

## Built-in Commands

### User Commands

| Command | Description |
|---|---|
| `/help [command]` | Show help for all or a specific command |
| `/clear` | Clear the current conversation |
| `/exit` | Exit Synerix |
| `/model <name>` | Switch to a model preset |
| `/model list` | Show available model presets |
| `/mode <name>` | Switch coding mode: `act`, `vibe`, `chat`, `architect`, `plan` |
| `/mcp list` | List configured MCP servers |
| `/skill` | Manage skills and sources |

## Creating Commands

### Basic Command

```rust
use async_trait::async_trait;
use synerix::command::{Command, CommandContext, CommandResult};

pub struct GreetCommand;

#[async_trait]
impl Command for GreetCommand {
    fn name(&self) -> &'static str {
        "greet"
    }

    fn description(&self) -> &'static str {
        "Greet someone by name"
    }

    fn usage(&self) -> &'static str {
        "greet <name>"
    }

    async fn execute(&self, ctx: CommandContext) -> CommandResult {
        let name = ctx.args().first()
            .ok_or("Please provide a name")?;

        ctx.reply(&format!("Hello, {name}!")).await;
        CommandResult::Success
    }
}
```

### Registering a Command

```rust
use synerix::app::App;

let mut app = App::new(config).build().await?;
app.command_registry()
   .register(Box::new(GreetCommand))
   .await?;
```

## Command Context

The `CommandContext` provides everything a command needs:

| Method | Returns | Description |
|---|---|---|
| `ctx.args()` | `&[String]` | Positional arguments |
| `ctx.has_flag(name)` | `bool` | Check for a flag (e.g. `--verbose`) |
| `ctx.get_option(name)` | `Option<&str>` | Get named option value |
| `ctx.reply(msg)` | `()` | Send a response message |
| `ctx.app()` | `&App` | Access the full application state |

## Command Result

```rust
pub enum CommandResult {
    Success,
    Error(String),
    NotImplemented,
}
```

## Lifecycle Hooks

```rust
#[async_trait]
impl Command for MyCommand {
    async fn before(&self, ctx: &CommandContext) -> CommandResult {
        // Runs before execute
        CommandResult::Success
    }

    async fn after(&self, ctx: &CommandContext, result: &CommandResult) {
        // Runs after execute, regardless of outcome
    }
}
```

## Next Steps

- [Configuration](/api/config) — Programmatic config access
- [Plugins](/api/plugins) — Package commands as plugins
