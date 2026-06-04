//! Application event types — event channels between agent, TUI, and app state.
// TODO: Agent event variants — some unused until multi-agent
#![allow(dead_code)]

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
    Error(String),
}
