//! Application event types — event channels between agent, TUI, and app state.

/// Agent events (sent from agent task to TUI)
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta(String),
    ToolCallStart {
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        name: String,
        output: String,
        is_error: bool,
    },
    Done,
    #[allow(dead_code)]
    Error(String),
}
