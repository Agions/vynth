# API Overview

Synerix exposes a Rust-native API for building plugins, custom commands, and integrations.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                     Synerix Core                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │   App    │  │  Event   │  │   Command        │  │
│  │  State   │◄─┤   Bus    │◄─┤   Registry       │  │
│  └────┬─────┘  └──────────┘  └──────────────────┘  │
│       │                                              │
│  ┌────┴─────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Renderer │  │  Agent   │  │   Plugin         │  │
│  │  (TUI)   │  │  Loop    │  │   Manager        │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Core Concepts

### App State

The `App` struct is the central state container. It holds chat history, agent state, configuration, and UI state.

```rust
use synerix::app::App;

let app = App::new(config)
    .with_agent(Box::new(MyAgent))
    .build()
    .await?;
```

### Event Bus

All internal communication flows through an event bus. Plugins and commands can publish and subscribe to events.

```rust
use synerix::event::{Event, EventBus};

event_bus.subscribe(|event| async move {
    match event {
        Event::MessageSent(msg) => { /* handle */ }
        Event::AgentStateChanged(state) => { /* handle */ }
        _ => {}
    }
});
```

### Command Registry

Extend Synerix with custom slash commands by registering against the command registry.

```rust
use synerix::command::CommandRegistry;

registry.register(Box::new(MyCommand)).await?;
```

## Next Steps

- [Commands](/api/commands) — Create custom slash commands
- [Configuration](/api/config) — Programmatic config access
- [Plugins](/api/plugins) — Build and distribute plugins
