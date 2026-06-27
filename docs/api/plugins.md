# Plugins

Extend Synerix with custom commands, themes, and external integrations.

## Plugin Types

| Type | Purpose |
|---|---|
| Command | Add slash commands |
| Skills | YAML/MD-based workflows |
| MCP | External tool servers (stdio / HTTP) |

## Creating Plugins

```rust
use async_trait::async_trait;
use synerix::plugin::{Plugin, PluginContext, PluginResult};

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    async fn initialize(&self, ctx: PluginContext) -> PluginResult {
        // Register commands, subscribe to events, etc.
        PluginResult::Success
    }

    async fn shutdown(&self) -> PluginResult {
        // Cleanup
        PluginResult::Success
    }
}
```

## Registering Plugins

```rust
use synerix::app::App;

let app = App::new(config)
    .with_plugin(Box::new(MyPlugin))
    .build()
    .await?;
```

## Plugin Context

```rust
pub struct PluginContext {
    pub config: Config,
    pub event_bus: EventBus,
    pub command_registry: CommandRegistry,
    pub skill_manager: SkillManager,
}
```

## Event Handling

Plugins can react to lifecycle events:

```rust
use synerix::event::{Event, EventFilter};

event_bus.subscribe(EventFilter::AgentState, |event| async move {
    // react to agent state changes
});
```

## Plugin Configuration

Plugins can ship their own config files:

```toml
# ~/.config/synerix/plugins/my-plugin.toml
[settings]
enabled = true
verbose = false

[options]
timeout = 30
retries = 3
```

## Distributing Plugins

- Publish as a Rust crate with a `synerix-plugin-*` prefix
- Ship a `skills/` directory alongside your codebase
- Register MCP servers via `config.toml`

## Next Steps

- [Commands](/api/commands) — Build commands that plugins can register
- [Configuration](/api/config) — Plugin config patterns
