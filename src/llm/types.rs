//! LLM unified types

use serde::{Deserialize, Serialize};

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String, // JSON string
}

/// Tool schema for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub function: FunctionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Chat response from LLM
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
}

/// Token usage statistics
#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Streaming chunk
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub delta: ChunkDelta,
}

#[derive(Debug, Clone)]
pub enum ChunkDelta {
    Text { content: String },
    ToolCall { id: String, name: String, args_delta: String },
    Done,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            name: None,
        }
    }

    /// Convert to JSON for API request
    pub fn to_json(&self) -> serde_json::Value {
        let mut json = serde_json::json!({
            "role": match self.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            }
        });

        if let Some(content) = &self.content {
            json["content"] = serde_json::json!(content);
        }

        if let Some(tool_calls) = &self.tool_calls {
            json["tool_calls"] = serde_json::json!(
                tool_calls
                    .iter()
                    .map(|tc| serde_json::json!({
                        "id": tc.id,
                        "type": "function",
                        "function": {
                            "name": tc.name,
                            "arguments": tc.arguments,
                        }
                    }))
                    .collect::<Vec<_>>()
            );
        }

        if let Some(tool_call_id) = &self.tool_call_id {
            json["tool_call_id"] = serde_json::json!(tool_call_id);
        }

        json
    }
}

impl ToolSchema {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.schema_type,
            "function": {
                "name": self.function.name,
                "description": self.function.description,
                "parameters": self.function.parameters,
            }
        })
    }
}
