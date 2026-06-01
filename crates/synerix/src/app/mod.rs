//! Application controller — global state machine + event dispatch
//!
//! Split into sub-modules:
//! - `state` — All struct/enum definitions + constructors
//! - `events` — AgentEvent and related event types
//! - `message` — ChatMessage, MessageRole, ToolCallDisplay
//! - `event_loop` — Event loop + agent/config event handling
//! - `input_handler` — Keyboard input handlers
//! - `actions` — Action execution (the big match block)
//! - `runner` — Application entry point

mod actions;
mod event_loop;
pub mod events;
mod input_handler;
pub mod message;
mod runner;
mod state;

// Re-export key types so external code sees the same API
pub use events::AgentEvent;
pub use message::{ChatMessage, MessageRole, ToolCallDisplay};
pub use runner::run;
pub use state::*;
