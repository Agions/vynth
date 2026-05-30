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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("Test Session", "gpt-4");
        assert_eq!(session.title, "Test Session");
        assert_eq!(session.model, "gpt-4");
        assert_eq!(session.total_tokens, 0);
        assert_eq!(session.message_count, 0);
        assert!(!session.id.is_empty());
    }

    #[test]
    fn test_session_timestamps() {
        let before = Utc::now();
        let session = Session::new("Test", "model");
        let after = Utc::now();
        assert!(session.created_at >= before);
        assert!(session.created_at <= after);
        assert_eq!(session.created_at, session.updated_at);
    }

    #[test]
    fn test_stored_message_user() {
        let msg = StoredMessage::user("session-1", "Hello!");
        assert_eq!(msg.session_id, "session-1");
        assert_eq!(msg.content, "Hello!");
        assert!(matches!(msg.role, StoredRole::User));
        assert!(msg.tool_calls.is_empty());
        assert_eq!(msg.tokens_used, 0);
        assert!(!msg.id.is_empty());
    }

    #[test]
    fn test_stored_message_assistant() {
        let msg = StoredMessage::assistant("session-1", "Hi there!");
        assert_eq!(msg.session_id, "session-1");
        assert_eq!(msg.content, "Hi there!");
        assert!(matches!(msg.role, StoredRole::Assistant));
    }

    #[test]
    fn test_stored_role_variants() {
        // Verify all variants can be constructed
        let _ = StoredRole::System;
        let _ = StoredRole::User;
        let _ = StoredRole::Assistant;
        let _ = StoredRole::Tool;
    }

    #[test]
    fn test_stored_tool_call() {
        let tc = StoredToolCall {
            id: "call-1".into(),
            name: "read_file".into(),
            arguments: "{\"path\": \"test.rs\"}".into(),
            result: Some("file contents".into()),
            is_error: false,
        };
        assert_eq!(tc.name, "read_file");
        assert!(!tc.is_error);
        assert!(tc.result.is_some());
    }

    #[test]
    fn test_stored_tool_call_error() {
        let tc = StoredToolCall {
            id: "call-2".into(),
            name: "shell_exec".into(),
            arguments: "{}".into(),
            result: Some("permission denied".into()),
            is_error: true,
        };
        assert!(tc.is_error);
    }

    #[test]
    fn test_uuid_v4_unique() {
        let id1 = uuid_v4();
        let id2 = uuid_v4();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }

    #[test]
    fn test_session_clone() {
        let session = Session::new("Test", "model");
        let cloned = session.clone();
        assert_eq!(cloned.id, session.id);
        assert_eq!(cloned.title, session.title);
    }

    #[test]
    fn test_stored_message_clone() {
        let msg = StoredMessage::user("s1", "content");
        let cloned = msg.clone();
        assert_eq!(cloned.id, msg.id);
        assert_eq!(cloned.content, msg.content);
    }
}
