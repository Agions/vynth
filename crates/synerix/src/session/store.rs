//! SQLite session store
//!
//! Uses `Mutex` instead of `RwLock` because `rusqlite::Connection` is `Send` but not `Sync`,
//! which means `RwLock<Connection>` (which requires `T: Sync` for shared reads) cannot be
//! wrapped in `Arc`. With WAL mode and `PRAGMA busy_timeout`, `Mutex` contention is minimal.

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::session::migration::run_migrations;
use crate::session::model::{Session, StoredMessage};
use synerix_core::types::role::Role as MessageRole;
use synerix_core::utils::datetime::parse_rfc3339_or_default;

/// SQLite-backed session store
pub struct SessionStore {
    conn: Arc<Mutex<Connection>>,
}

impl SessionStore {
    /// Acquire the connection lock, converting poison errors to AppError
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|e| AppError::MutexPoisoned(e.to_string()))
    }

    /// Open or create the session database
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        store.run_migrations()?;

        Ok(store)
    }

    /// Create an in-memory store (for testing)
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        store.run_migrations()?;

        Ok(store)
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        run_migrations(&conn)?;
        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, session: &Session) -> Result<(), AppError> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at, model, total_tokens, message_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                session.id,
                session.title,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.model,
                session.total_tokens,
                session.message_count,
            ],
        )?;
        Ok(())
    }

    /// List all sessions (most recent first)
    pub fn list_sessions(&self) -> Result<Vec<Session>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at, model, total_tokens, message_count
             FROM sessions ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: parse_rfc3339_or_default(&row.get::<_, String>(2)?),
                    updated_at: parse_rfc3339_or_default(&row.get::<_, String>(3)?),
                    model: row.get(4)?,
                    total_tokens: row.get(5)?,
                    message_count: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Save a message to a session
    pub fn save_message(&self, message: &StoredMessage) -> Result<(), AppError> {
        let conn = self.lock_conn()?;

        let role_str = match message.role {
            MessageRole::System => MessageRole::System.as_str(),
            MessageRole::User => MessageRole::User.as_str(),
            MessageRole::Assistant => MessageRole::Assistant.as_str(),
            MessageRole::Tool => MessageRole::Tool.as_str(),
        };

        let tool_calls_json = serde_json::to_string(&message.tool_calls).unwrap_or_default();

        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, tool_calls, timestamp, tokens_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                message.id,
                message.session_id,
                role_str,
                message.content,
                tool_calls_json,
                message.timestamp.to_rfc3339(),
                message.tokens_used,
            ],
        )?;

        // Update session stats
        conn.execute(
            "UPDATE sessions SET updated_at = ?1, message_count = message_count + 1,
             total_tokens = total_tokens + ?2 WHERE id = ?3",
            rusqlite::params![
                chrono::Utc::now().to_rfc3339(),
                message.tokens_used,
                message.session_id,
            ],
        )?;

        Ok(())
    }

    /// Load all messages for a session
    pub fn load_messages(&self, session_id: &str) -> Result<Vec<StoredMessage>, AppError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, tool_calls, timestamp, tokens_used
             FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC",
        )?;

        let messages = stmt
            .query_map([session_id], |row| {
                let role_str: String = row.get(2)?;
            let role = match role_str.as_str() {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "tool" => MessageRole::Tool,
                _ => MessageRole::User,
            };

                let tool_calls_json: String = row.get(4)?;
                let tool_calls = serde_json::from_str(&tool_calls_json).unwrap_or_default();

                Ok(StoredMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role,
                    content: row.get(3)?,
                    tool_calls,
                    timestamp: parse_rfc3339_or_default(&row.get::<_, String>(5)?),
                    tokens_used: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::{Session, StoredMessage};
    use synerix_core::types::role::Role as MessageRole;

    fn test_store() -> SessionStore {
        SessionStore::memory().expect("create memory store")
    }

    #[test]
    fn test_open_memory_store() {
        let store = test_store();
        let sessions = store.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_create_and_list_sessions() {
        let store = test_store();
        let session = Session::new("Test Session", "gpt-4");
        store.create_session(&session).unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].title, "Test Session");
        assert_eq!(sessions[0].model, "gpt-4");
    }

    #[test]
    fn test_create_multiple_sessions() {
        let store = test_store();
        store.create_session(&Session::new("S1", "m1")).unwrap();
        store.create_session(&Session::new("S2", "m2")).unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_save_and_load_messages() {
        let store = test_store();
        let session = Session::new("Test", "gpt-4");
        store.create_session(&session).unwrap();

        let msg = StoredMessage::user(&session.id, "Hello!");
        store.save_message(&msg).unwrap();

        let messages = store.load_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
        assert!(matches!(messages[0].role, MessageRole::User));
    }

    #[test]
    fn test_save_multiple_messages() {
        let store = test_store();
        let session = Session::new("Test", "gpt-4");
        store.create_session(&session).unwrap();

        store
            .save_message(&StoredMessage::user(&session.id, "Q1"))
            .unwrap();
        store
            .save_message(&StoredMessage::assistant(&session.id, "A1"))
            .unwrap();
        store
            .save_message(&StoredMessage::user(&session.id, "Q2"))
            .unwrap();

        let messages = store.load_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 3);
        assert!(matches!(messages[0].role, MessageRole::User));
        assert!(matches!(messages[1].role, MessageRole::Assistant));
        assert!(matches!(messages[2].role, MessageRole::User));
    }

    #[test]
    fn test_load_messages_empty_session() {
        let store = test_store();
        let session = Session::new("Empty", "gpt-4");
        store.create_session(&session).unwrap();

        let messages = store.load_messages(&session.id).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_session_stats_updated_on_message() {
        let store = test_store();
        let session = Session::new("Test", "gpt-4");
        store.create_session(&session).unwrap();

        let mut msg = StoredMessage::user(&session.id, "Hello");
        msg.tokens_used = 10;
        store.save_message(&msg).unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions[0].message_count, 1);
        assert_eq!(sessions[0].total_tokens, 10);
    }

    #[test]
    fn test_open_file_store() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).unwrap();
        store
            .create_session(&Session::new("File Session", "m1"))
            .unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(db_path.exists());
    }
}
