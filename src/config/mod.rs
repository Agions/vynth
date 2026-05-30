//! Configuration management

pub mod settings;
pub mod keymap;
pub mod watcher;

pub use settings::{
    LlmConfig, McpServerConfig, McpTransport, Provider, SandboxMode, Settings,
};
pub use watcher::{spawn_config_watcher, ConfigReload};
pub use keymap::{Action, KeyBindings, KeymapProfile};
