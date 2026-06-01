//! Configuration management

pub mod keymap;
pub mod settings;
pub mod config_watcher;

pub use keymap::{Action, KeyBindings, KeymapProfile};
pub use settings::{
    InlineAgentConfig, LlmConfig, McpServerConfig, McpTransport, Provider, SandboxMode, Settings,
    SkillSourceConfig,
};
pub use config_watcher::{spawn_config_watcher, ConfigReload};
