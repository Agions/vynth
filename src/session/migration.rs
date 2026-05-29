//! Database schema migrations

use rusqlite::Connection;
use crate::error::AppError;

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            model TEXT NOT NULL,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            message_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            tool_calls TEXT NOT NULL DEFAULT '[]',
            timestamp TEXT NOT NULL,
            tokens_used INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);
        "
    )?;

    tracing::info!("Database migrations complete");
    Ok(())
}
