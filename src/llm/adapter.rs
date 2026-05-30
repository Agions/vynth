//! LLM Provider adapter trait

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
}

/// Build OpenAI-compatible request body
fn build_request_body(
    model: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
    stream: bool,
) -> serde_json::Value {
    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| m.to_json()).collect();

    let mut body = serde_json::json!({
        "model": model,
        "messages": msgs,
        "stream": stream,
    });

    if !tools.is_empty() {
        let tool_schemas: Vec<serde_json::Value> = tools.iter().map(|t| t.to_json()).collect();
        body["tools"] = serde_json::json!(tool_schemas);
    }

    body
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
