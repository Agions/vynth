//! Unified error types for Syncode

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
