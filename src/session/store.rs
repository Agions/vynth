//! SQLite session store

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::session::migration::run_migrations;
use crate::session::model::{Session, StoredMessage, StoredRole};

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
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .unwrap_or_default()
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap_or_default()
                        .with_timezone(&chrono::Utc),
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
            StoredRole::System => "system",
            StoredRole::User => "user",
            StoredRole::Assistant => "assistant",
            StoredRole::Tool => "tool",
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
                    "system" => StoredRole::System,
                    "user" => StoredRole::User,
                    "assistant" => StoredRole::Assistant,
                    "tool" => StoredRole::Tool,
                    _ => StoredRole::User,
                };

                let tool_calls_json: String = row.get(4)?;
                let tool_calls = serde_json::from_str(&tool_calls_json).unwrap_or_default();

                Ok(StoredMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role,
                    content: row.get(3)?,
                    tool_calls,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap_or_default()
                        .with_timezone(&chrono::Utc),
                    tokens_used: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }
}
