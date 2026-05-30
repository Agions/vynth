//! Configuration management

pub mod keymap;
pub mod settings;
pub mod watcher;

pub use keymap::{Action, KeyBindings, KeymapProfile};
pub use settings::{
    InlineAgentConfig, LlmConfig, McpServerConfig, McpTransport, Provider, SandboxMode,
    Settings, SkillSourceConfig,
};
pub use watcher::{spawn_config_watcher, ConfigReload};
