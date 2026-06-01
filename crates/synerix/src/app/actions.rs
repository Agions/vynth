//! Action execution — the big match block for all user actions.

use crate::app::state::{App, InputMode};
use crate::config::keymap::Action;
use crate::error::AppError;

impl App {
    /// Execute a resolved Action
    pub(crate) async fn execute_action(&mut self, action: Action) -> Result<(), AppError> {
        match action {
            // Text editing
            Action::InsertChar(c) => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.dirty_flags.input = true;
            }
            Action::DeleteChar => {
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
                self.dirty_flags.input = true;
            }
            Action::DeleteCharForward => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_buffer.replace_range(self.input_cursor..next, "");
                }
                self.dirty_flags.input = true;
            }
            Action::DeleteWord => {
                // Delete word backwards
                if self.input_cursor > 0 {
                    let before = &self.input_buffer[..self.input_cursor];
                    let trimmed = before.trim_end();
                    let new_pos = trimmed
                        .rfind(|c: char| c.is_whitespace())
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    self.input_buffer
                        .replace_range(new_pos..self.input_cursor, "");
                    self.input_cursor = new_pos;
                }
                self.dirty_flags.input = true;
            }
            Action::KillToEnd => {
                self.input_buffer.truncate(self.input_cursor);
                self.dirty_flags.input = true;
            }
            Action::KillToStart => {
                self.yank_buffer = self.input_buffer[..self.input_cursor].to_string();
                self.input_buffer.replace_range(..self.input_cursor, "");
                self.input_cursor = 0;
                self.dirty_flags.input = true;
            }
            Action::MoveCursorLeft => {
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_cursor = prev;
                }
                self.dirty_flags.input = true;
            }
            Action::MoveCursorRight => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_cursor = next;
                }
                self.dirty_flags.input = true;
            }
            Action::MoveCursorHome => {
                self.input_cursor = 0;
                self.dirty_flags.input = true;
            }
            Action::MoveCursorEnd => {
                self.input_cursor = self.input_buffer.len();
                self.dirty_flags.input = true;
            }

            // Mode transitions
            Action::SubmitMessage => {
                self.submit_message();
                self.dirty_flags.chat = true;
                self.dirty_flags.input = true;
            }
            Action::EnterInsertMode => {
                self.mode = InputMode::Insert;
                self.dirty_flags.input = true;
            }
            Action::EnterInsertModeAppend => {
                // Move cursor right one char, then enter insert
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_cursor = next;
                }
                self.mode = InputMode::Insert;
                self.dirty_flags.input = true;
            }
            Action::EnterInsertModeOpenLineBelow => {
                // Move to end, add newline, enter insert
                self.input_cursor = self.input_buffer.len();
                self.input_buffer.push('\n');
                self.input_cursor = self.input_buffer.len();
                self.mode = InputMode::Insert;
                self.dirty_flags.input = true;
            }
            Action::EnterInsertModeOpenLineAbove => {
                // Add newline at current position, enter insert
                self.input_buffer.insert(self.input_cursor, '\n');
                // Cursor stays at the inserted newline position
                self.mode = InputMode::Insert;
                self.dirty_flags.input = true;
            }
            Action::EnterNormalMode => {
                self.mode = InputMode::Normal;
                // Move cursor back one if possible (vim convention)
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_cursor = prev;
                }
                self.dirty_flags.input = true;
            }
            Action::EnterCommandMode => {
                self.mode = InputMode::Command;
                self.dirty_flags.input = true;
            }
            Action::EnterSearchMode => {
                self.mode = InputMode::Search;
                self.dirty_flags.input = true;
            }

            // Scrolling
            Action::ScrollUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
                self.dirty_flags.chat = true;
            }
            Action::ScrollDown => {
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
                self.dirty_flags.chat = true;
            }
            Action::ScrollToBottom => {
                self.chat_state.scroll_offset = 0;
                self.dirty_flags.chat = true;
            }
            Action::ScrollPageUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_add(10);
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset > max_scroll {
                    self.chat_state.scroll_offset = max_scroll;
                }
                self.dirty_flags.chat = true;
            }
            Action::ScrollPageDown => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(10);
                self.dirty_flags.chat = true;
            }

            // Application
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Cancel => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
                self.dirty_flags.input = true;
            }
            Action::TabNext => {
                use super::state::SidebarTab;
                // Cycle sidebar tabs
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Sessions,
                    SidebarTab::Sessions => SidebarTab::Skills,
                    SidebarTab::Skills => SidebarTab::Files,
                };
                self.dirty_flags.sidebar = true;
            }
            Action::TabPrev => {
                use super::state::SidebarTab;
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Skills,
                    SidebarTab::Sessions => SidebarTab::Files,
                    SidebarTab::Skills => SidebarTab::Sessions,
                };
                self.dirty_flags.sidebar = true;
            }

            // Yank/paste
            Action::YankLine => {
                self.yank_buffer = self.input_buffer.clone();
            }
            Action::Paste => {
                let paste = self.yank_buffer.clone();
                self.input_buffer.insert_str(self.input_cursor, &paste);
                self.input_cursor += paste.len();
                self.dirty_flags.input = true;
            }

            // Vim clear line (dd)
            Action::ClearLine => {
                self.yank_buffer = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.dirty_flags.input = true;
            }

            Action::Noop => {}
        }
        Ok(())
    }
}
