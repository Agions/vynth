//! Application settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::AppError;

/// Top-level application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// LLM provider configuration
    pub llm: LlmConfig,
    /// TUI preferences
    pub ui: UiConfig,
    /// Sandbox security settings
    pub sandbox: SandboxConfig,
    /// MCP server configurations
    #[serde(default)]
    pub mcp: Vec<McpServerConfig>,
    /// Skills directory
    pub skills_dir: Option<PathBuf>,
    /// External skill sources (git repos, URLs, additional directories)
    #[serde(default)]
    pub skill_sources: Vec<SkillSourceConfig>,
    /// Custom agents directory
    pub agents_dir: Option<PathBuf>,
    /// Custom agent definitions (inline in config)
    #[serde(default)]
    pub agents: Vec<InlineAgentConfig>,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider type: deepseek, mimo, custom
    pub provider: Provider,
    /// API key
    pub api_key: String,
    /// API base URL (overrides provider default)
    pub base_url: Option<String>,
    /// Model identifier
    pub model: String,
    /// Max context window (tokens)
    pub context_window: usize,
    /// Max output tokens
    pub max_output_tokens: usize,
    /// Temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_temperature() -> f32 {
    0.7
}

/// Supported LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    DeepSeek,
    MiMo,
    Custom { base_url: String },
}

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme: dark, light
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Keymap profile: vim, emacs, default
    #[serde(default = "default_keymap")]
    pub keymap: String,
    /// Show line numbers in diff
    #[serde(default = "default_true")]
    pub diff_line_numbers: bool,
    /// Streaming typing delay (ms)
    #[serde(default = "default_typing_delay")]
    pub typing_delay_ms: u64,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_keymap() -> String {
    "default".to_string()
}

fn default_true() -> bool {
    true
}

fn default_typing_delay() -> u64 {
    10
}

/// Sandbox security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Sandbox mode: auto, confirm, preview_only
    #[serde(default = "default_sandbox_mode")]
    pub mode: SandboxMode,
    /// Enable atomic file writes
    #[serde(default = "default_true")]
    pub atomic_writes: bool,
}

fn default_sandbox_mode() -> SandboxMode {
    SandboxMode::Confirm
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    /// Auto-execute all tools
    Auto,
    /// Confirm high-risk operations
    Confirm,
    /// Preview only, never execute
    PreviewOnly,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (unique identifier)
    pub name: String,
    /// Transport type
    pub transport: McpTransport,
    /// Allowed tool name patterns (glob). Empty = allow all.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Environment variables for the MCP server process
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory for the MCP server process
    pub cwd: Option<PathBuf>,
    /// Auto-reconnect on disconnect
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,
    /// Connection timeout in seconds
    #[serde(default = "default_mcp_timeout")]
    pub timeout_secs: u64,
}

fn default_mcp_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String },
}

/// External skill source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSourceConfig {
    /// Source type: local, git, url
    #[serde(rename = "type")]
    pub source_type: String,
    /// Path or URL
    pub location: String,
    /// Optional branch for git sources
    pub branch: Option<String>,
    /// Include patterns
    #[serde(default)]
    pub include: Vec<String>,
    /// Exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Inline custom agent definition in config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineAgentConfig {
    /// Agent name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// System prompt
    pub system_prompt: String,
    /// Allowed tools
    #[serde(default)]
    pub tools: Vec<String>,
    /// Max turns
    #[serde(default = "default_agent_turns")]
    pub max_turns: usize,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_agent_turns() -> usize {
    10
}

impl Settings {
    /// Load settings from config file, with environment variable overrides
    ///
    /// NOTE: Uses synchronous `std::fs::read_to_string` intentionally.
    /// Config loading happens once at startup and the file is small,
    /// so async I/O is unnecessary here.
    pub fn load() -> Result<Self, AppError> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| AppError::Config(format!("Failed to read config: {}", e)))?;
            let mut settings: Settings = toml::from_str(&content)?;
            settings.apply_env_overrides();
            Ok(settings)
        } else {
            tracing::info!("No config file found, using defaults");
            Ok(Self::defaults())
        }
    }

    /// Default config path: ~/.config/synerix/config.toml
    fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("synerix")
            .join("config.toml")
    }

    /// Create default settings
    fn defaults() -> Self {
        Self {
            llm: LlmConfig {
                provider: Provider::DeepSeek,
                api_key: String::new(),
                base_url: None,
                model: "deepseek-chat".to_string(),
                context_window: 128_000,
                max_output_tokens: 8192,
                temperature: 0.7,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                keymap: "default".to_string(),
                diff_line_numbers: true,
                typing_delay_ms: 10,
            },
            sandbox: SandboxConfig {
                mode: SandboxMode::Confirm,
                atomic_writes: true,
            },
            mcp: Vec::new(),
            skills_dir: None,
            skill_sources: Vec::new(),
            agents_dir: None,
            agents: Vec::new(),
        }
    }

    /// Override settings with environment variables
    fn apply_env_overrides(&mut self) {
        if let Ok(key) = std::env::var("SYNERIX_API_KEY") {
            self.llm.api_key = key;
        }
        if let Ok(url) = std::env::var("SYNERIX_BASE_URL") {
            self.llm.base_url = Some(url);
        }
        if let Ok(model) = std::env::var("SYNERIX_MODEL") {
            self.llm.model = model;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::defaults();
        assert_eq!(settings.llm.model, "deepseek-chat");
        assert_eq!(settings.llm.temperature, 0.7);
        assert_eq!(settings.ui.theme, "dark");
        assert_eq!(settings.ui.keymap, "default");
        assert!(settings.ui.diff_line_numbers);
        assert_eq!(settings.ui.typing_delay_ms, 10);
        assert_eq!(settings.sandbox.mode, SandboxMode::Confirm);
        assert!(settings.sandbox.atomic_writes);
        assert!(settings.mcp.is_empty());
        assert!(settings.agents.is_empty());
    }

    #[test]
    fn test_parse_toml_config() {
        let toml = r#"
[llm]
provider = "deepseek"
api_key = "sk-test"
model = "deepseek-chat"
context_window = 128000
max_output_tokens = 8192

[ui]
theme = "light"
keymap = "vim"

[sandbox]
mode = "auto"
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.llm.api_key, "sk-test");
        assert_eq!(settings.llm.model, "deepseek-chat");
        assert_eq!(settings.ui.theme, "light");
        assert_eq!(settings.ui.keymap, "vim");
        assert_eq!(settings.sandbox.mode, SandboxMode::Auto);
    }

    #[test]
    fn test_parse_toml_with_mcp() {
        let toml = r#"
[llm]
provider = "deepseek"
api_key = "sk-test"
model = "deepseek-chat"
context_window = 128000
max_output_tokens = 8192

[sandbox]

[ui]

[[mcp]]
name = "filesystem"
transport = { type = "stdio", command = "mcp-fs", args = [] }
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.mcp.len(), 1);
        assert_eq!(settings.mcp[0].name, "filesystem");
        assert_eq!(settings.mcp[0].timeout_secs, 30); // default
        assert!(settings.mcp[0].auto_reconnect); // default
    }

    #[test]
    fn test_parse_toml_with_agents() {
        let toml = r#"
[llm]
provider = "deepseek"
api_key = "sk-test"
model = "deepseek-chat"
context_window = 128000
max_output_tokens = 8192

[sandbox]

[ui]

[[agents]]
name = "reviewer"
system_prompt = "You are a code reviewer"
max_turns = 5
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.agents.len(), 1);
        assert_eq!(settings.agents[0].name, "reviewer");
        assert_eq!(settings.agents[0].max_turns, 5);
    }

    #[test]
    fn test_provider_variants() {
        let toml = r#"
[llm]
provider = "mimo"
api_key = "key"
model = "mimo-7b"
context_window = 32000
max_output_tokens = 4096

[sandbox]

[ui]
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert!(matches!(settings.llm.provider, Provider::MiMo));
    }

    #[test]
    fn test_sandbox_mode_variants() {
        let toml_auto = r#"
[llm]
provider = "deepseek"
api_key = "k"
model = "m"
context_window = 1000
max_output_tokens = 1000

[sandbox]
mode = "auto"

[ui]
"#;
        let s: Settings = toml::from_str(toml_auto).unwrap();
        assert_eq!(s.sandbox.mode, SandboxMode::Auto);

        let toml_preview = toml_auto.replace("mode = \"auto\"", "mode = \"preview_only\"");
        let s: Settings = toml::from_str(&toml_preview).unwrap();
        assert_eq!(s.sandbox.mode, SandboxMode::PreviewOnly);
    }

    #[test]
    fn test_mcp_transport_types() {
        let toml = r#"
[llm]
provider = "deepseek"
api_key = "k"
model = "m"
context_window = 1000
max_output_tokens = 1000

[sandbox]

[ui]

[[mcp]]
name = "stdio-server"
transport = { type = "stdio", command = "server", args = ["--port", "3000"] }

[[mcp]]
name = "http-server"
transport = { type = "http", url = "http://localhost:8080" }
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.mcp.len(), 2);
        assert!(matches!(
            settings.mcp[0].transport,
            McpTransport::Stdio { .. }
        ));
        assert!(matches!(
            settings.mcp[1].transport,
            McpTransport::Http { .. }
        ));
    }

    #[test]
    fn test_default_temperature() {
        assert_eq!(default_temperature(), 0.7);
    }

    #[test]
    fn test_default_theme() {
        assert_eq!(default_theme(), "dark");
    }

    #[test]
    fn test_default_keymap() {
        assert_eq!(default_keymap(), "default");
    }

    #[test]
    fn test_default_sandbox_mode() {
        assert_eq!(default_sandbox_mode(), SandboxMode::Confirm);
    }

    #[test]
    fn test_default_mcp_timeout() {
        assert_eq!(default_mcp_timeout(), 30);
    }

    #[test]
    fn test_default_agent_turns() {
        assert_eq!(default_agent_turns(), 10);
    }

    #[test]
    fn test_skill_source_config() {
        let toml = r#"
[llm]
provider = "deepseek"
api_key = "k"
model = "m"
context_window = 1000
max_output_tokens = 1000

[sandbox]

[ui]

[[skill_sources]]
type = "git"
location = "https://github.com/example/skills"
branch = "main"
include = ["*.yaml"]
exclude = ["test_*"]
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.skill_sources.len(), 1);
        assert_eq!(settings.skill_sources[0].source_type, "git");
        assert_eq!(settings.skill_sources[0].branch, Some("main".into()));
    }
}
