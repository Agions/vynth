//! Settings loading and environment override logic

use std::path::PathBuf;

use crate::error::AppError;

use super::{
    InlineAgentConfig, LlmConfig, McpServerConfig, Provider, SandboxConfig, SandboxMode, Settings,
    SkillSourceConfig, UiConfig,
};

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
    pub(crate) fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("synerix")
            .join("config.toml")
    }

    /// Create default settings
    pub(crate) fn defaults() -> Self {
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
    pub(crate) fn apply_env_overrides(&mut self) {
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
