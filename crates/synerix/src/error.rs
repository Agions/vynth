//! Unified error types for Synerix
//!
//! Application-wide error enum with conversions from standard library types.

use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("MCP permission denied: server={server}, tool={tool}")]
    McpPermissionDenied { server: String, tool: String },

    #[error("MCP transport error: {0}")]
    McpTransport(String),

    #[error("Max turns exceeded: {0}")]
    MaxTurnsExceeded(usize),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[allow(dead_code)]
    #[error("Plugin event partial failure: {failed_count}/{total_count} plugins failed")]
    PluginEventPartialFailure {
        failed_count: usize,
        total_count: usize,
    },

    #[allow(dead_code)]
    #[error("Plugin init partial failure: {failed_count}/{total_count} plugin(s) failed")]
    PluginInitPartialFailure {
        failed_count: usize,
        total_count: usize,
    },
}

/// Convert channel send errors
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AppError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        AppError::ChannelSend(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_config() {
        let err = AppError::Config("bad setting".into());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("bad setting"));
    }

    #[test]
    fn test_error_display_llm() {
        let err = AppError::Llm("rate limited".into());
        assert!(err.to_string().contains("LLM error"));
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn test_error_display_tool_not_found() {
        let err = AppError::ToolNotFound("git".into());
        assert!(err.to_string().contains("Tool not found"));
        assert!(err.to_string().contains("git"));
    }

    #[test]
    fn test_error_display_mcp_permission() {
        let err = AppError::McpPermissionDenied {
            server: "fs".into(),
            tool: "write".into(),
        };
        let s = err.to_string();
        assert!(s.contains("fs"));
        assert!(s.contains("write"));
        assert!(s.contains("permission denied"));
    }

    #[test]
    fn test_error_display_max_turns() {
        let err = AppError::MaxTurnsExceeded(50);
        assert!(err.to_string().contains("50"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(app_err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_display_execution_failed() {
        let err = AppError::ExecutionFailed("boom".into());
        assert!(err.to_string().contains("Execution failed"));
    }
}
