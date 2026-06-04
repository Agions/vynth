//! Settings loading and environment override logic

use std::path::PathBuf;

use crate::error::AppError;

use super::{LlmConfig, Provider, SandboxConfig, SandboxMode, Settings, UiConfig};

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
                model: "deepseek-v4-flash".to_string(),
                context_window: 128_000,
                max_output_tokens: 8192,
                temperature: 0.7,
                system_prompt_tokens: 2000,
                tools_schema_tokens: 3000,
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
                tool_timeout_secs: 120,
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
        // Expanded env var substitution: support ${VAR} syntax in config values
        if self.llm.api_key.starts_with("${") && self.llm.api_key.ends_with('}') {
            let var_name = &self.llm.api_key[2..self.llm.api_key.len() - 1];
            if let Ok(val) = std::env::var(var_name) {
                self.llm.api_key = val;
            }
        }

        // Direct env var overrides (highest priority)
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

    /// Save settings back to the config file
    pub fn save(&self) -> Result<(), AppError> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create config dir: {}", e)))?;
        }
        let toml = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(&config_path, toml)
            .map_err(|e| AppError::Config(format!("Failed to write config: {}", e)))?;
        tracing::info!("Settings saved to {:?}", config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_settings_defaults() {
        let s = Settings::defaults();
        assert_eq!(s.llm.model, "deepseek-v4-flash");
        assert_eq!(s.llm.context_window, 128_000);
        assert_eq!(s.llm.max_output_tokens, 8192);
        assert_eq!(s.llm.temperature, 0.7);
        assert_eq!(s.ui.theme, "dark");
        assert_eq!(s.ui.keymap, "default");
        assert!(s.ui.diff_line_numbers);
        assert_eq!(s.ui.typing_delay_ms, 10);
        assert_eq!(s.sandbox.mode, SandboxMode::Confirm);
        assert!(s.sandbox.atomic_writes);
        assert!(s.mcp.is_empty());
        assert!(s.skills_dir.is_none());
        assert!(s.skill_sources.is_empty());
        assert!(s.agents_dir.is_none());
        assert!(s.agents.is_empty());
    }

    #[test]
    fn config_path_ends_with_synerix_config() {
        let path = Settings::config_path();
        assert!(path.ends_with("synerix/config.toml"));
    }

    #[test]
    #[serial]
    fn apply_env_overrides_api_key() {
        let key = "test_api_key_12345";
        std::env::set_var("SYNERIX_API_KEY", key);
        let mut s = Settings::defaults();
        s.apply_env_overrides();
        assert_eq!(s.llm.api_key, key);
        std::env::remove_var("SYNERIX_API_KEY");
    }

    #[test]
    #[serial]
    fn apply_env_overrides_base_url() {
        std::env::set_var("SYNERIX_BASE_URL", "http://localhost:9999");
        let mut s = Settings::defaults();
        s.apply_env_overrides();
        assert_eq!(s.llm.base_url, Some("http://localhost:9999".to_string()));
        std::env::remove_var("SYNERIX_BASE_URL");
    }

    #[test]
    #[serial]
    fn apply_env_overrides_model() {
        std::env::set_var("SYNERIX_MODEL", "custom-model-v2");
        let mut s = Settings::defaults();
        s.apply_env_overrides();
        assert_eq!(s.llm.model, "custom-model-v2");
        std::env::remove_var("SYNERIX_MODEL");
    }

    #[test]
    #[serial]
    fn apply_env_overrides_no_env_keeps_defaults() {
        std::env::remove_var("SYNERIX_API_KEY");
        std::env::remove_var("SYNERIX_BASE_URL");
        std::env::remove_var("SYNERIX_MODEL");
        let mut s = Settings::defaults();
        let orig_key = s.llm.api_key.clone();
        let orig_model = s.llm.model.clone();
        s.apply_env_overrides();
        assert_eq!(s.llm.api_key, orig_key);
        assert_eq!(s.llm.model, orig_model);
        assert!(s.llm.base_url.is_none());
    }
}
