//! Keyboard input handlers — insert, normal, command, search modes.

use super::state::{App, InputMode};
use crate::error::AppError;

impl App {
    /// Dispatch key event based on current input mode.
    pub(crate) async fn handle_mode_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match self.mode {
            InputMode::Insert => self.handle_insert_key(key).await,
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Command => self.handle_command_key(key).await,
            InputMode::Search => self.handle_search_key(key).await,
        }
    }

    async fn handle_insert_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                self.submit_message();
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
            }
            crossterm::event::KeyCode::Delete => {
                self.delete_char_after_cursor();
            }
            crossterm::event::KeyCode::Left => {
                self.move_cursor_left();
            }
            crossterm::event::KeyCode::Right => {
                self.move_cursor_right();
            }
            crossterm::event::KeyCode::Home => {
                self.input_cursor = 0;
            }
            crossterm::event::KeyCode::End => {
                self.input_cursor = self.input_buffer.len();
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Char('i') => {
                self.mode = InputMode::Insert;
            }
            crossterm::event::KeyCode::Char(':') => {
                self.mode = InputMode::Command;
            }
            crossterm::event::KeyCode::Char('/') => {
                self.mode = InputMode::Search;
            }
            crossterm::event::KeyCode::Char('q') => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Char('j') => {
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
            }
            crossterm::event::KeyCode::Char('k') => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
            }
            crossterm::event::KeyCode::Char('G') => {
                self.chat_state.scroll_offset = 0;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.clear_input();
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                tracing::debug!("Command: {}", self.input_buffer);
                self.clear_input();
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.clear_input();
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                tracing::debug!("Search: {}", self.input_buffer);
                self.clear_input();
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            _ => {}
        }
        Ok(())
    }

    // ── Cursor helpers ───────────────────────────────────────

    fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor = self.prev_char_pos();
        }
    }

    fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor = self.next_char_pos();
        }
    }

    fn delete_char_before_cursor(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.prev_char_pos();
            self.input_buffer.replace_range(prev..self.input_cursor, "");
            self.input_cursor = prev;
        }
    }

    fn delete_char_after_cursor(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            let next = self.next_char_pos();
            self.input_buffer.replace_range(self.input_cursor..next, "");
        }
    }
}
