//! Chat transcript scrolling.
//!
//! Consolidates the chat scroll arithmetic that was previously duplicated
//! across `actions` (`Scroll*`), `input_handler` (normal-mode `j`/`k`/`G`) and
//! `event_loop` (mouse wheel). All call sites now go through these primitives.
//!
//! `chat_state.scroll_offset` measures distance *back from the latest message*:
//! `0` is the bottom (newest), larger values reveal older history. The methods
//! are named by that semantic — `older`/`newer` — rather than visual up/down,
//! because the existing key/mouse/action bindings disagree on which direction
//! "up" means; naming by offset direction keeps each binding unambiguous.

use super::state::App;

impl App {
    /// Scroll toward older messages by `lines`, clamped to the oldest message.
    pub(crate) fn scroll_chat_older(&mut self, lines: usize) {
        let max_scroll = self.chat_state.messages.len().saturating_sub(1);
        self.chat_state.scroll_offset = (self.chat_state.scroll_offset + lines).min(max_scroll);
        self.dirty_flags.chat = true;
    }

    /// Scroll toward newer messages by `lines`, clamped to the bottom.
    pub(crate) fn scroll_chat_newer(&mut self, lines: usize) {
        self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(lines);
        self.dirty_flags.chat = true;
    }

    /// Jump to the latest message (bottom of the transcript).
    pub(crate) fn scroll_chat_to_bottom(&mut self) {
        self.chat_state.scroll_offset = 0;
        self.dirty_flags.chat = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::App;
    use crate::app::{ChatMessage, MessageRole};
    use crate::config::Settings;

    fn app_with_messages(count: usize) -> App {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new_with_settings(Settings::defaults(), rx);
        app.chat_state.messages = (0..count)
            .map(|_| ChatMessage {
                role: MessageRole::System,
                content: String::new(),
                tool_calls: Vec::new(),
            })
            .collect();
        app
    }

    #[test]
    fn older_clamps_to_oldest_message() {
        let mut app = app_with_messages(5);
        app.scroll_chat_older(10);
        assert_eq!(app.chat_state.scroll_offset, 4); // messages.len() - 1
    }

    #[test]
    fn newer_saturates_at_bottom() {
        let mut app = app_with_messages(5);
        app.scroll_chat_older(3);
        app.scroll_chat_newer(10);
        assert_eq!(app.chat_state.scroll_offset, 0);
    }

    #[test]
    fn to_bottom_resets_offset() {
        let mut app = app_with_messages(5);
        app.scroll_chat_older(2);
        app.scroll_chat_to_bottom();
        assert_eq!(app.chat_state.scroll_offset, 0);
    }
}
