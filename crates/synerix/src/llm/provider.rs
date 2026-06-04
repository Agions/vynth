//! LLM Provider factory
// TODO: Provider factory — not yet wired into agent loop
#![allow(dead_code)]

use crate::config::{LlmConfig, Provider};
use crate::llm::adapter::{LlmAdapter, OpenAICompatAdapter};

/// Create an LLM adapter from configuration
pub fn create_provider(config: &LlmConfig) -> Box<dyn LlmAdapter> {
    match &config.provider {
        Provider::DeepSeek => {
            let base_url = config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
            Box::new(OpenAICompatAdapter::new(
                &base_url,
                &config.api_key,
                &config.model,
                config.context_window,
            ))
        }
        Provider::MiMo => {
            let base_url = config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.xiaomi.com/v1".to_string());
            Box::new(OpenAICompatAdapter::new(
                &base_url,
                &config.api_key,
                &config.model,
                config.context_window,
            ))
        }
        Provider::Custom { base_url } => Box::new(OpenAICompatAdapter::new(
            base_url,
            &config.api_key,
            &config.model,
            config.context_window,
        )),
    }
}
