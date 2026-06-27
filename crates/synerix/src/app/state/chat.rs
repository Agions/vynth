//! Chat conversation state types.

use super::super::message::ChatMessage;

/// Chat conversation state
#[derive(Debug, Clone, Default)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub streaming_text: String,
    pub is_streaming: bool,
    /// Number of lines scrolled up from the bottom (0 = latest at bottom)
    pub scroll_offset: usize,
}
