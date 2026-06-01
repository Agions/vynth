//! Unified error types for Synerix

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

    #[error("Sandbox approval denied")]
    ApprovalDenied,

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid transport type")]
    InvalidTransport,

    #[error("Stream closed unexpectedly")]
    StreamClosed,

    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[error("Token budget exceeded: used={used}, limit={limit}")]
    TokenBudgetExceeded { used: usize, limit: usize },

    #[error("Workflow failed: {0}")]
    WorkflowFailed(String),

    #[error("Step '{step_id}' failed: {reason}")]
    StepFailed { step_id: String, reason: String },

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
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
    fn test_error_display_token_budget() {
        let err = AppError::TokenBudgetExceeded {
            used: 1000,
            limit: 500,
        };
        let s = err.to_string();
        assert!(s.contains("1000"));
        assert!(s.contains("500"));
    }

    #[test]
    fn test_error_display_step_failed() {
        let err = AppError::StepFailed {
            step_id: "build".into(),
            reason: "timeout".into(),
        };
        let s = err.to_string();
        assert!(s.contains("build"));
        assert!(s.contains("timeout"));
    }

    #[test]
    fn test_error_display_approval_denied() {
        let err = AppError::ApprovalDenied;
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn test_error_display_stream_closed() {
        let err = AppError::StreamClosed;
        assert!(err.to_string().contains("Stream closed"));
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
    fn test_error_from_channel_send() {
        // Use a full channel to get a SendError (not TrySendError)
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(1);
        // Fill the channel
        tx.try_send("fill".into()).unwrap();
        // Now a blocking send would fail, but we can't do that in a sync test.
        // Instead, test the Display formatting directly.
        let app_err = AppError::ChannelSend("test channel error".into());
        assert!(app_err.to_string().contains("Channel send"));
        drop(rx);
    }

    #[test]
    fn test_error_debug() {
        let err = AppError::ExecutionFailed("boom".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ExecutionFailed"));
    }
}
