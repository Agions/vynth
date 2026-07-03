//! Keyboard input handlers — insert, normal, command, search modes.

use super::state::{App, DirtyFlags, InputMode};
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
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Up => {
                if self.move_slash_selection(-1) {
                    self.dirty_flags.insert(DirtyFlags::INPUT);
                }
            }
            crossterm::event::KeyCode::Down => {
                if self.move_slash_selection(1) {
                    self.dirty_flags.insert(DirtyFlags::INPUT);
                }
            }
            crossterm::event::KeyCode::Enter => {
                if !self.apply_selected_slash_command() {
                    self.submit_message();
                }
                self.dirty_flags.insert(DirtyFlags::INPUT);
                self.dirty_flags.insert(DirtyFlags::CHAT);
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
                self.reset_slash_selection();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Delete => {
                self.delete_char_after_cursor();
                self.reset_slash_selection();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Left => {
                self.move_cursor_left();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Right => {
                self.move_cursor_right();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Home => {
                self.input_cursor = 0;
                self.reset_slash_selection();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::End => {
                self.input_cursor = self.input_buffer.len();
                self.reset_slash_selection();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.reset_slash_selection();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Char('i') => {
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Char(':') => {
                self.mode = InputMode::Command;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Char('/') => {
                self.input_buffer.clear();
                self.input_buffer.push('/');
                self.input_cursor = self.input_buffer.len();
                self.reset_slash_selection();
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Char('q') => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Char('j') => {
                self.scroll_chat_older(1);
            }
            crossterm::event::KeyCode::Char('k') => {
                self.scroll_chat_newer(1);
            }
            crossterm::event::KeyCode::Char('G') => {
                self.scroll_chat_to_bottom();
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
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Enter => {
                tracing::debug!("{}: {}", debug_label, self.input_buffer);
                self.clear_input();
                self.mode = InputMode::Normal;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Backspace => {
                self.delete_char_before_cursor();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            crossterm::event::KeyCode::Char(c)
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            _ => {}
        }
        Ok(())
    }

    // ── Cursor helpers ───────────────────────────────────────

    /// Handle key events when approval popup is showing
    async fn handle_approval_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        use crate::sandbox::approval::ApprovalDecision;
        match key.code {
            crossterm::event::KeyCode::Char('y') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::Allow).await;
                }
                self.pending_approval = None;
                self.dirty_flags.insert(DirtyFlags::APPROVAL);
            }
            crossterm::event::KeyCode::Char('n') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::Deny).await;
                }
                self.pending_approval = None;
                self.dirty_flags.insert(DirtyFlags::APPROVAL);
            }
            crossterm::event::KeyCode::Char('a') => {
                if let Some(tx) = self.approval_decision_tx.take() {
                    let _ = tx.send(ApprovalDecision::AllowAlways).await;
                }
                self.pending_approval = None;
                self.dirty_flags.insert(DirtyFlags::APPROVAL);
            }
            _ => {}
        }
        Ok(())
    }

    fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.reset_slash_selection();
    }

    fn move_slash_selection(&mut self, delta: isize) -> bool {
        let matches = crate::slash::menu::menu_matches(&self.input_buffer);
        if matches.is_empty() {
            return false;
        }

        let len = matches.len();
        let selected = self.slash_menu_state.selected.min(len.saturating_sub(1));
        self.slash_menu_state.selected = if delta < 0 {
            selected.checked_sub(1).unwrap_or(len - 1)
        } else {
            (selected + 1) % len
        };
        true
    }

    fn apply_selected_slash_command(&mut self) -> bool {
        let matches = crate::slash::menu::menu_matches(&self.input_buffer);
        if matches.is_empty() {
            return false;
        }

        let selected = self.slash_menu_state.selected.min(matches.len() - 1);
        self.input_buffer = matches[selected].name.to_string();
        self.input_cursor = self.input_buffer.len();
        self.reset_slash_selection();
        true
    }

    fn reset_slash_selection(&mut self) {
        self.slash_menu_state.selected = 0;
    }
}
