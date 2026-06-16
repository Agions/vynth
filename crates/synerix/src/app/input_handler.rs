//! Keyboard input handlers — insert, normal, command, search modes.

use super::state::{App, InputMode};
use crate::error::AppError;

impl App {
    /// Dispatch key event based on current input mode.
    /// Intercepts approval popup keys (y/n/a) before mode dispatch.
    pub(crate) async fn handle_mode_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        // Check for pending approval first
        if self.pending_approval.is_some() {
            return self.handle_approval_key(key).await;
        }
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
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Enter => {
                self.submit_message();
                self.dirty_flags.input = true;
                self.dirty_flags.chat = true;
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Delete => {
                self.delete_char_after_cursor();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Left => {
                self.move_cursor_left();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Right => {
                self.move_cursor_right();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Home => {
                self.input_cursor = 0;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::End => {
                self.input_cursor = self.input_buffer.len();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.dirty_flags.input = true;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Char('i') => {
                self.mode = InputMode::Insert;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Char(':') => {
                self.mode = InputMode::Command;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Char('/') => {
                self.mode = InputMode::Search;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Char('q') => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Char('j') => {
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
                self.dirty_flags.chat = true;
            }
            crossterm::event::KeyCode::Char('k') => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
                self.dirty_flags.chat = true;
            }
            crossterm::event::KeyCode::Char('G') => {
                self.chat_state.scroll_offset = 0;
                self.dirty_flags.chat = true;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        self.handle_common_key(key, "Command").await
    }

    async fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        self.handle_common_key(key, "Search").await
    }

    /// Shared key handling for command/search modes: Esc, Enter, Backspace, regular chars.
    async fn handle_common_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        debug_label: &str,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.clear_input();
                self.mode = InputMode::Normal;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Enter => {
                tracing::debug!("{}: {}", debug_label, self.input_buffer);
                self.clear_input();
                self.mode = InputMode::Normal;
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
                self.dirty_flags.input = true;
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.dirty_flags.input = true;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Cursor helpers ───────────────────────────────────────
    
    /// Handle key events when approval popup is showing
    async fn handle_approval_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        use crate::sandbox::approval::ApprovalDecision;
        match key.code {
            crossterm::event::KeyCode::Char('y') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::Allow).await;
                }
                self.pending_approval = None;
                self.dirty_flags.approval = true;
            }
            crossterm::event::KeyCode::Char('n') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::Deny).await;
                }
                self.pending_approval = None;
                self.dirty_flags.approval = true;
            }
            crossterm::event::KeyCode::Char('a') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::AllowAlways).await;
                }
                self.pending_approval = None;
                self.dirty_flags.approval = true;
            }
            _ => {}
        }
        Ok(())
    }

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
