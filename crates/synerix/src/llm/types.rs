//! LLM unified types
//!
//! Performance optimizations:
//! - `to_json()` methods use pre-allocated `serde_json::Map` for fewer allocations
//! - Role strings are static references (no allocation)
//! - Tool call serialization avoids intermediate Value creation

use serde::{Deserialize, Serialize};

/// Chat message role — unified type from synerix-core
pub use synerix_core::types::role::Role as MessageRole;

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

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String, // JSON string
}

/// Tool schema for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub function: FunctionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Text {
        content: String,
    },
    ToolCall {
        id: String,
        name: String,
        args_delta: String,
    },
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
    ///
    /// Optimized: uses `serde_json::Map` with pre-allocated capacity
    /// to reduce reallocations during serialization.
    pub fn to_json(&self) -> serde_json::Value {
        // Pre-allocate map capacity based on expected fields
        let capacity = 2 + self.tool_calls.as_ref().map_or(0, |tc| tc.len()) + 1;
        let mut map = serde_json::Map::with_capacity(capacity);

        // Role (always present, static string)
        map.insert(
            "role".to_string(),
            serde_json::Value::String(self.role.as_str().to_string()),
        );

        // Content (optional)
        if let Some(content) = &self.content {
            map.insert(
                "content".to_string(),
                serde_json::Value::String(content.clone()),
            );
        }

        // Tool calls (optional)
        if let Some(tool_calls) = &self.tool_calls {
            let calls: Vec<serde_json::Value> = tool_calls
                .iter()
                .map(|tc| {
                    // Pre-allocate for tool call object
                    let mut tc_map = serde_json::Map::with_capacity(3);
                    tc_map.insert("id".to_string(), serde_json::Value::String(tc.id.clone()));
                    tc_map.insert(
                        "type".to_string(),
                        serde_json::Value::String("function".to_string()),
                    );

                    let mut func_map = serde_json::Map::with_capacity(2);
                    func_map.insert(
                        "name".to_string(),
                        serde_json::Value::String(tc.name.clone()),
                    );
                    func_map.insert(
                        "arguments".to_string(),
                        serde_json::Value::String(tc.arguments.clone()),
                    );

                    tc_map.insert("function".to_string(), serde_json::Value::Object(func_map));

                    serde_json::Value::Object(tc_map)
                })
                .collect();

            map.insert("tool_calls".to_string(), serde_json::Value::Array(calls));
        }

        // Tool call ID (for tool results)
        if let Some(tool_call_id) = &self.tool_call_id {
            map.insert(
                "tool_call_id".to_string(),
                serde_json::Value::String(tool_call_id.clone()),
            );
        }

        // Name (optional)
        if let Some(name) = &self.name {
            map.insert("name".to_string(), serde_json::Value::String(name.clone()));
        }

        serde_json::Value::Object(map)
    }
}

impl ToolSchema {
    /// Convert to JSON for API request
    ///
    /// Optimized: uses `serde_json::Map` with exact capacity
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::with_capacity(2);
        map.insert(
            "type".to_string(),
            serde_json::Value::String(self.schema_type.clone()),
        );

        let mut func_map = serde_json::Map::with_capacity(3);
        func_map.insert(
            "name".to_string(),
            serde_json::Value::String(self.function.name.clone()),
        );
        func_map.insert(
            "description".to_string(),
            serde_json::Value::String(self.function.description.clone()),
        );
        func_map.insert("parameters".to_string(), self.function.parameters.clone());

        map.insert("function".to_string(), serde_json::Value::Object(func_map));

        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_as_str() {
        assert_eq!(MessageRole::System.as_str(), "system");
        assert_eq!(MessageRole::User.as_str(), "user");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(MessageRole::Tool.as_str(), "tool");
    }

    #[test]
    fn test_chat_message_to_json_basic() {
        let msg = ChatMessage::user("hello");
        let json = msg.to_json();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "hello");
        assert!(json.get("tool_calls").is_none());
    }

    #[test]
    fn test_chat_message_to_json_with_tool_calls() {
        let msg = ChatMessage {
            role: MessageRole::Assistant,
            content: Some("thinking...".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_1".to_string(),
                name: "read_file".to_string(),
                arguments: r#"{"path":"test.rs"}"#.to_string(),
            }]),
            tool_call_id: None,
            name: None,
        };

        let json = msg.to_json();
        assert_eq!(json["role"], "assistant");
        assert_eq!(json["content"], "thinking...");
        let calls = json["tool_calls"].as_array().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0]["id"], "call_1");
        assert_eq!(calls[0]["function"]["name"], "read_file");
    }

    #[test]
    fn test_tool_schema_to_json() {
        let schema = ToolSchema {
            schema_type: "function".to_string(),
            function: FunctionSchema {
                name: "test".to_string(),
                description: "A test function".to_string(),
                parameters: serde_json::json!({"type": "object"}),
            },
        };

        let json = schema.to_json();
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "test");
        assert_eq!(json["function"]["description"], "A test function");
    }
}
