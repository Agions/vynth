//! Action execution — the big match block for all user actions.

use crate::app::state::{App, InputMode, MessageRole};
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
            }
            Action::DeleteChar => {
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
            }
            Action::DeleteCharForward => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_buffer.replace_range(self.input_cursor..next, "");
                }
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
            }
            Action::KillToEnd => {
                self.input_buffer.truncate(self.input_cursor);
            }
            Action::KillToStart => {
                self.yank_buffer = self.input_buffer[..self.input_cursor].to_string();
                self.input_buffer.replace_range(..self.input_cursor, "");
                self.input_cursor = 0;
            }
            Action::MoveCursorLeft => {
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_cursor = prev;
                }
            }
            Action::MoveCursorRight => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_cursor = next;
                }
            }
            Action::MoveCursorHome => {
                self.input_cursor = 0;
            }
            Action::MoveCursorEnd => {
                self.input_cursor = self.input_buffer.len();
            }

            // Mode transitions
            Action::SubmitMessage => {
                self.submit_message();
            }
            Action::EnterInsertMode => {
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeAppend => {
                // Move cursor right one char, then enter insert
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.next_char_pos();
                    self.input_cursor = next;
                }
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeOpenLineBelow => {
                // Move to end, add newline, enter insert
                self.input_cursor = self.input_buffer.len();
                self.input_buffer.push('\n');
                self.input_cursor = self.input_buffer.len();
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeOpenLineAbove => {
                // Add newline at current position, enter insert
                self.input_buffer.insert(self.input_cursor, '\n');
                // Cursor stays at the inserted newline position
                self.mode = InputMode::Insert;
            }
            Action::EnterNormalMode => {
                self.mode = InputMode::Normal;
                // Move cursor back one if possible (vim convention)
                if self.input_cursor > 0 {
                    let prev = self.prev_char_pos();
                    self.input_cursor = prev;
                }
            }
            Action::EnterCommandMode => {
                self.mode = InputMode::Command;
            }
            Action::EnterSearchMode => {
                self.mode = InputMode::Search;
            }

            // Scrolling
            Action::ScrollUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
            }
            Action::ScrollDown => {
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
            }
            Action::ScrollToBottom => {
                self.chat_state.scroll_offset = 0;
            }
            Action::ScrollPageUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_add(10);
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset > max_scroll {
                    self.chat_state.scroll_offset = max_scroll;
                }
            }
            Action::ScrollPageDown => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(10);
            }

            // Application
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Cancel => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            Action::TabNext => {
                use super::state::SidebarTab;
                // Cycle sidebar tabs
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Sessions,
                    SidebarTab::Sessions => SidebarTab::Skills,
                    SidebarTab::Skills => SidebarTab::Files,
                };
            }
            Action::TabPrev => {
                use super::state::SidebarTab;
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Skills,
                    SidebarTab::Sessions => SidebarTab::Files,
                    SidebarTab::Skills => SidebarTab::Sessions,
                };
            }

            // Yank/paste
            Action::YankLine => {
                self.yank_buffer = self.input_buffer.clone();
            }
            Action::Paste => {
                let paste = self.yank_buffer.clone();
                self.input_buffer.insert_str(self.input_cursor, &paste);
                self.input_cursor += paste.len();
            }

            // Vim clear line (dd)
            Action::ClearLine => {
                self.yank_buffer = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
            }

            Action::Noop => {}
        }
        Ok(())
    }
}
