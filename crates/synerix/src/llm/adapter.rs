//! LLM Provider adapter trait
// TODO: Infrastructure awaiting main-loop integration
#![allow(dead_code)]

use futures::stream::Stream;
use std::pin::Pin;

use crate::error::AppError;
use crate::llm::types::{ChatMessage, ChatResponse, StreamChunk, ToolSchema};

/// LLM Provider unified abstraction
/// All models (DeepSeek V4, MiMo-v2.5, custom) implement this trait
#[async_trait::async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Non-streaming inference: returns complete response
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSchema],
    ) -> Result<ChatResponse, AppError>;

    /// Streaming inference: returns AsyncStream<Item=StreamChunk>
    /// Core performance requirement: first token latency < 50ms
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSchema],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>, AppError>;

    /// Model identifier (e.g. "deepseek-v4", "mimo-v2.5")
    fn model_id(&self) -> &str;

    /// Context window size (for token budget calculation)
    fn context_window(&self) -> usize;

    /// Provider name (e.g. "deepseek", "openai", "custom")
    fn provider_name(&self) -> &'static str {
        "unknown"
    }

    /// Estimate token count for text (default: ~4 chars/token for English, ~2 for CJK)
    /// Providers may override with more accurate counting (e.g., tiktoken)
    fn count_tokens(&self, text: &str) -> usize {
        crate::token_estimator::estimate_tokens(text)
    }
}

/// OpenAI-compatible API adapter
/// Works with DeepSeek, MiMo, and any OpenAI-compatible endpoint
pub struct OpenAICompatAdapter {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    context_window_size: usize,
}

impl OpenAICompatAdapter {
    pub fn new(base_url: &str, api_key: &str, model: &str, context_window: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            context_window_size: context_window,
        }
    }
}

#[async_trait::async_trait]
impl LlmAdapter for OpenAICompatAdapter {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSchema],
    ) -> Result<ChatResponse, AppError> {
        let body = build_request_body(&self.model, messages, tools, false);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        parse_chat_response(&json)
    }

    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSchema],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>, AppError> {
        let body = build_request_body(&self.model, messages, tools, true);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("API error {}: {}", status, text)));
        }

        let byte_stream = response.bytes_stream();
        let s = crate::llm::stream::parse_sse_stream(byte_stream);

        Ok(Box::pin(s))
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn context_window(&self) -> usize {
        self.context_window_size
    }

    fn provider_name(&self) -> &'static str {
        "openai_compat"
    }
}

/// Build OpenAI-compatible request body
fn build_request_body(
    model: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
    stream: bool,
) -> serde_json::Value {
    let capacity = if tools.is_empty() { 3 } else { 4 };
    let mut map = serde_json::Map::with_capacity(capacity);

    let mut msgs = Vec::with_capacity(messages.len());
    for m in messages {
        msgs.push(m.to_json());
    }
    map.insert("messages".into(), serde_json::Value::Array(msgs));

    map.insert("model".into(), serde_json::Value::String(model.into()));
    map.insert("stream".into(), serde_json::Value::Bool(stream));

    if !tools.is_empty() {
        let mut tool_schemas = Vec::with_capacity(tools.len());
        for t in tools {
            tool_schemas.push(t.to_json());
        }
        map.insert("tools".into(), serde_json::Value::Array(tool_schemas));
    }

    serde_json::Value::Object(map)
}

/// Parse a non-streaming chat response
fn parse_chat_response(json: &serde_json::Value) -> Result<ChatResponse, AppError> {
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let tool_calls = json["choices"][0]["message"]["tool_calls"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|tc| {
                    Some(crate::llm::types::ToolCall {
                        id: tc["id"].as_str()?.to_string(),
                        name: tc["function"]["name"].as_str()?.to_string(),
                        arguments: tc["function"]["arguments"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let usage = json["usage"].as_object().map(|u| crate::llm::types::Usage {
        prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as usize,
        completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as usize,
        total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as usize,
    });

    Ok(ChatResponse {
        content,
        tool_calls,
        usage,
    })
}
