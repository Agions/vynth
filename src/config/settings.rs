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

    /// Default config path: ~/.config/syncode/config.toml
    fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncode")
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
        if let Ok(key) = std::env::var("SYNCODE_API_KEY") {
            self.llm.api_key = key;
        }
        if let Ok(url) = std::env::var("SYNCODE_BASE_URL") {
            self.llm.base_url = Some(url);
        }
        if let Ok(model) = std::env::var("SYNCODE_MODEL") {
            self.llm.model = model;
        }
    }
}
