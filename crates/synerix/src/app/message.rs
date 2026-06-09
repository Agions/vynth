//! Message types — ChatMessage, MessageRole, ToolCallDisplay

/// A single chat message in the TUI conversation view
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Vec<ToolCallDisplay>,
}

/// Chat message role for TUI — unified type from synerix-core
pub use synerix_core::types::role::Role as MessageRole;

/// Tool call display info for the TUI
#[derive(Debug, Clone)]
pub struct ToolCallDisplay {
    pub name: String,
    pub args_preview: String,
    pub result: Option<String>,
    pub is_error: bool,
}
