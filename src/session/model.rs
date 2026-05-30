//! Session and message models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub total_tokens: usize,
    pub message_count: usize,
}

/// A single message in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub session_id: String,
    pub role: StoredRole,
    pub content: String,
    pub tool_calls: Vec<StoredToolCall>,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoredRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A tool call stored in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub is_error: bool,
}

impl Session {
    pub fn new(title: &str, model: &str) -> Self {
        let now = Utc::now();
        Self {
            id: uuid_v4(),
            title: title.to_string(),
            created_at: now,
            updated_at: now,
            model: model.to_string(),
            total_tokens: 0,
            message_count: 0,
        }
    }
}

impl StoredMessage {
    pub fn user(session_id: &str, content: &str) -> Self {
        Self {
            id: uuid_v4(),
            session_id: session_id.to_string(),
            role: StoredRole::User,
            content: content.to_string(),
            tool_calls: Vec::new(),
            timestamp: Utc::now(),
            tokens_used: 0,
        }
    }

    pub fn assistant(session_id: &str, content: &str) -> Self {
        Self {
            id: uuid_v4(),
            session_id: session_id.to_string(),
            role: StoredRole::Assistant,
            content: content.to_string(),
            tool_calls: Vec::new(),
            timestamp: Utc::now(),
            tokens_used: 0,
        }
    }
}

/// Simple UUID v4 generator (no external dependency)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!("{:032x}", timestamp)
}
