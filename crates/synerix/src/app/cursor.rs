//! Cursor and character navigation over the input buffer.
//!
//! Consolidates the character-boundary and cursor-movement logic that was
//! previously duplicated across `state::app` (`prev_char_pos`/`next_char_pos`),
//! `input_handler` (`move_cursor_*`/`delete_char_*`) and inlined in `actions`.
//! All methods operate on `input_buffer` + `input_cursor` and are UTF-8 safe
//! (they step by character boundaries, so CJK and multibyte input never split).

use super::state::App;

impl App {
    /// Byte position of the previous character boundary before the cursor.
    pub(crate) fn prev_char_pos(&self) -> usize {
        self.input_buffer[..self.input_cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Byte position of the next character boundary after the cursor.
    pub(crate) fn next_char_pos(&self) -> usize {
        self.input_buffer[self.input_cursor..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.input_cursor + i)
            .unwrap_or(self.input_buffer.len())
    }

    /// Move the cursor one character left, clamped to the start.
    pub(crate) fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor = self.prev_char_pos();
        }
    }

    /// Move the cursor one character right, clamped to the end.
    pub(crate) fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor = self.next_char_pos();
        }
    }

    /// Delete the character before the cursor (Backspace).
    pub(crate) fn delete_char_before_cursor(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.prev_char_pos();
            self.input_buffer.replace_range(prev..self.input_cursor, "");
            self.input_cursor = prev;
        }
    }

    /// Delete the character after the cursor (Delete).
    pub(crate) fn delete_char_after_cursor(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            let next = self.next_char_pos();
            self.input_buffer.replace_range(self.input_cursor..next, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::App;
    use crate::config::Settings;

    fn app_with_input(buffer: &str, cursor: usize) -> App {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new_with_settings(Settings::defaults(), rx);
        app.input_buffer = buffer.to_string();
        app.input_cursor = cursor;
        app
    }

    #[test]
    fn navigates_multibyte_input_by_char_boundary() {
        // "你好" — each CJK char is 3 bytes.
        let mut app = app_with_input("你好", 0);
        app.move_cursor_right();
        assert_eq!(app.input_cursor, 3);
        app.move_cursor_right();
        assert_eq!(app.input_cursor, 6);
        app.move_cursor_right();
        assert_eq!(app.input_cursor, 6, "clamped at end");
        app.move_cursor_left();
        assert_eq!(app.input_cursor, 3);
    }

    #[test]
    fn delete_before_and_after_respect_char_boundaries() {
        let mut app = app_with_input("a你b", 4); // cursor after "你"
        app.delete_char_before_cursor();
        assert_eq!(app.input_buffer, "ab");
        assert_eq!(app.input_cursor, 1);
        app.delete_char_after_cursor();
        assert_eq!(app.input_buffer, "a");
        assert_eq!(app.input_cursor, 1);
    }
}
