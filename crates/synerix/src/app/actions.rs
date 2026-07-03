//! Action execution — the big match block for all user actions.

use crate::app::state::{App, DirtyFlags, InputMode};
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
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::DeleteChar => {
                self.delete_char_before_cursor();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::DeleteCharForward => {
                self.delete_char_after_cursor();
                self.dirty_flags.insert(DirtyFlags::INPUT);
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
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::KillToEnd => {
                self.input_buffer.truncate(self.input_cursor);
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::KillToStart => {
                self.yank_buffer = self.input_buffer[..self.input_cursor].to_string();
                self.input_buffer.replace_range(..self.input_cursor, "");
                self.input_cursor = 0;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::MoveCursorLeft => {
                self.move_cursor_left();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::MoveCursorRight => {
                self.move_cursor_right();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::MoveCursorHome => {
                self.input_cursor = 0;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::MoveCursorEnd => {
                self.input_cursor = self.input_buffer.len();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }

            // Mode transitions
            Action::SubmitMessage => {
                self.submit_message();
                self.dirty_flags.insert(DirtyFlags::CHAT);
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterInsertMode => {
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterInsertModeAppend => {
                // Move cursor right one char, then enter insert
                self.move_cursor_right();
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterInsertModeOpenLineBelow => {
                // Move to end, add newline, enter insert
                self.input_cursor = self.input_buffer.len();
                self.input_buffer.push('\n');
                self.input_cursor = self.input_buffer.len();
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterInsertModeOpenLineAbove => {
                // Add newline at current position, enter insert
                self.input_buffer.insert(self.input_cursor, '\n');
                // Cursor stays at the inserted newline position
                self.mode = InputMode::Insert;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterNormalMode => {
                self.mode = InputMode::Normal;
                // Move cursor back one if possible (vim convention)
                self.move_cursor_left();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterCommandMode => {
                self.mode = InputMode::Command;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::OpenSlashCommand => {
                self.input_buffer.clear();
                self.input_buffer.push('/');
                self.input_cursor = self.input_buffer.len();
                self.mode = InputMode::Insert;
                self.focused_panel = crate::app::FocusedPanel::Input;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::EnterSearchMode => {
                self.mode = InputMode::Search;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }

            // Scrolling
            Action::ScrollUp => {
                self.scroll_chat_newer(1);
            }
            Action::ScrollDown => {
                self.scroll_chat_older(1);
            }
            Action::ScrollToBottom => {
                self.scroll_chat_to_bottom();
            }
            Action::ScrollPageUp => {
                self.scroll_chat_older(10);
            }
            Action::ScrollPageDown => {
                self.scroll_chat_newer(10);
            }

            // Application
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Cancel => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }
            Action::TabNext => {
                self.coding_mode = self.coding_mode.next();
                self.dirty_flags.insert(DirtyFlags::CHAT);
                self.dirty_flags.insert(DirtyFlags::INPUT);
                self.dirty_flags.insert(DirtyFlags::STATUS);
            }
            Action::TabPrev => {
                self.coding_mode = self.coding_mode.previous();
                self.dirty_flags.insert(DirtyFlags::CHAT);
                self.dirty_flags.insert(DirtyFlags::INPUT);
                self.dirty_flags.insert(DirtyFlags::STATUS);
            }

            // Yank/paste
            Action::YankLine => {
                self.yank_buffer = self.input_buffer.clone();
            }
            Action::Paste => {
                let paste = self.yank_buffer.clone();
                self.input_buffer.insert_str(self.input_cursor, &paste);
                self.input_cursor += paste.len();
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }

            // Vim clear line (dd)
            Action::ClearLine => {
                self.yank_buffer = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.dirty_flags.insert(DirtyFlags::INPUT);
            }

            Action::Noop => {}
        }
        Ok(())
    }
}
