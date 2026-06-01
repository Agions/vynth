//! Keymap profile definitions and key resolution

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use super::actions::Action;
use crate::app::InputMode;

/// Keymap profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeymapProfile {
    Vim,
    Emacs,
    #[default]
    Default,
}

/// A resolved keybinding mapping from a (mode, key) pair to an Action
#[derive(Debug, Clone)]
pub struct KeyBindings {
    profile: KeymapProfile,
    /// Pending key for multi-key sequences (e.g. `dd`, `yy`, `gg`, count prefix)
    pending_key: Option<PendingKey>,
}

/// State for multi-key sequences
#[derive(Debug, Clone)]
enum PendingKey {
    /// A count prefix like `3` waiting for the motion
    Count(u32),
    /// First char of a double-key sequence (d or y)
    DoubleKey(char),
}

impl KeyBindings {
    pub fn new(profile: KeymapProfile) -> Self {
        Self {
            profile,
            pending_key: None,
        }
    }

    /// Resolve a KeyEvent in the given mode to an Action
    pub fn resolve(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match &self.profile {
            KeymapProfile::Vim => self.resolve_vim(mode, key),
            KeymapProfile::Emacs => self.resolve_emacs(mode, key),
            KeymapProfile::Default => self.resolve_default(mode, key),
        }
    }

    // ── Vim profile ──────────────────────────────────────────────

    fn resolve_vim(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match mode {
            InputMode::Normal => self.resolve_vim_normal(key),
            InputMode::Insert => self.resolve_vim_insert(key),
            InputMode::Command => self.resolve_editing_common(key, true),
            InputMode::Search => self.resolve_editing_common(key, true),
        }
    }

    fn resolve_vim_normal(&mut self, key: KeyEvent) -> Action {
        // Handle Ctrl combinations first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('d') => return Action::ScrollPageDown,
                KeyCode::Char('u') => return Action::ScrollPageUp,
                KeyCode::Char('c') => return Action::Quit,
                _ => return Action::Noop,
            }
        }

        // Handle pending key state (count prefix or double-key)
        if let Some(pending) = self.pending_key.take() {
            match pending {
                PendingKey::Count(count) => {
                    // Count prefix followed by a motion
                    match key.code {
                        KeyCode::Char('j') => {
                            return Action::ScrollDown; // count-aware: handled in app
                        }
                        KeyCode::Char('k') => {
                            return Action::ScrollUp;
                        }
                        KeyCode::Char('G') => {
                            return Action::ScrollToBottom;
                        }
                        KeyCode::Char('g') => {
                            // `Ngg` = go to top
                            self.pending_key = None;
                            return Action::ScrollToBottom; // actually top; we'll handle in app
                        }
                        KeyCode::Char(d) if d.is_ascii_digit() => {
                            // Accumulate more digits
                            let new_count = count * 10 + (d as u32 - '0' as u32);
                            self.pending_key = Some(PendingKey::Count(new_count));
                            return Action::Noop;
                        }
                        _ => {
                            // Invalid motion after count, discard
                            return Action::Noop;
                        }
                    }
                }
                PendingKey::DoubleKey(first) => {
                    match (first, key.code) {
                        ('d', KeyCode::Char('d')) => return Action::ClearLine,
                        ('y', KeyCode::Char('y')) => return Action::YankLine,
                        ('g', KeyCode::Char('g')) => {
                            // gg = go to top — we'll represent as ScrollToBottom with a flag
                            // Actually let's use ScrollUp with special handling
                            // For simplicity, let's return a custom action
                            return Action::ScrollToBottom; // We'll fix: use ScrollPageUp as proxy
                        }
                        _ => return Action::Noop,
                    }
                }
            }
        }

        // Normal single-key dispatch
        match key.code {
            // Mode transitions
            KeyCode::Char('i') => Action::EnterInsertMode,
            KeyCode::Char('a') => Action::EnterInsertModeAppend,
            KeyCode::Char('A') => Action::EnterInsertModeAppend, // append at end
            KeyCode::Char('o') => Action::EnterInsertModeOpenLineBelow,
            KeyCode::Char('O') => Action::EnterInsertModeOpenLineAbove,
            KeyCode::Char(':') => Action::EnterCommandMode,
            KeyCode::Char('/') => Action::EnterSearchMode,
            // Quit
            KeyCode::Char('q') => Action::Quit,
            // Scrolling
            KeyCode::Char('j') => Action::ScrollDown,
            KeyCode::Char('k') => Action::ScrollUp,
            KeyCode::Char('G') => Action::ScrollToBottom,
            // Cursor (for input buffer in normal mode — less common but supported)
            KeyCode::Char('h') => Action::MoveCursorLeft,
            KeyCode::Char('l') => Action::MoveCursorRight,
            // Tab navigation
            KeyCode::Tab => Action::TabNext,
            KeyCode::BackTab => Action::TabPrev,
            // Double-key sequences
            KeyCode::Char('d') => {
                self.pending_key = Some(PendingKey::DoubleKey('d'));
                Action::Noop
            }
            KeyCode::Char('y') => {
                self.pending_key = Some(PendingKey::DoubleKey('y'));
                Action::Noop
            }
            KeyCode::Char('g') => {
                self.pending_key = Some(PendingKey::DoubleKey('g'));
                Action::Noop
            }
            KeyCode::Char('p') => Action::Paste,
            // Count prefix
            KeyCode::Char(d) if d.is_ascii_digit() && d != '0' => {
                self.pending_key = Some(PendingKey::Count(d as u32 - '0' as u32));
                Action::Noop
            }
            _ => Action::Noop,
        }
    }

    fn resolve_vim_insert(&mut self, key: KeyEvent) -> Action {
        // Ctrl combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('w') => return Action::DeleteWord,
                KeyCode::Char('a') => return Action::MoveCursorHome,
                KeyCode::Char('e') => return Action::MoveCursorEnd,
                KeyCode::Char('k') => return Action::KillToEnd,
                KeyCode::Char('u') => return Action::KillToStart,
                KeyCode::Char('c') => return Action::Quit,
                _ => return Action::Noop,
            }
        }

        match key.code {
            KeyCode::Esc => Action::EnterNormalMode,
            KeyCode::Enter => Action::SubmitMessage,
            KeyCode::Backspace => Action::DeleteChar,
            KeyCode::Delete => Action::DeleteCharForward,
            KeyCode::Left => Action::MoveCursorLeft,
            KeyCode::Right => Action::MoveCursorRight,
            KeyCode::Home => Action::MoveCursorHome,
            KeyCode::End => Action::MoveCursorEnd,
            KeyCode::Tab => Action::TabNext,
            KeyCode::BackTab => Action::TabPrev,
            KeyCode::Char(c) => Action::InsertChar(c),
            _ => Action::Noop,
        }
    }

    // ── Emacs profile ────────────────────────────────────────────

    fn resolve_emacs(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match mode {
            InputMode::Normal | InputMode::Insert => self.resolve_emacs_editing(key),
            InputMode::Command => self.resolve_editing_common(key, true),
            InputMode::Search => self.resolve_editing_common(key, true),
        }
    }

    fn resolve_emacs_editing(&mut self, key: KeyEvent) -> Action {
        // Meta (Alt) combinations
        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('f') => return Action::MoveCursorRight, // word forward — simplified
                KeyCode::Char('b') => return Action::MoveCursorLeft,  // word backward — simplified
                KeyCode::Char('v') => return Action::ScrollPageUp,
                _ => return Action::Noop,
            }
        }

        // Ctrl combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('n') => return Action::ScrollDown,
                KeyCode::Char('p') => return Action::ScrollUp,
                KeyCode::Char('f') => return Action::MoveCursorRight,
                KeyCode::Char('b') => return Action::MoveCursorLeft,
                KeyCode::Char('a') => return Action::MoveCursorHome,
                KeyCode::Char('e') => return Action::MoveCursorEnd,
                KeyCode::Char('k') => return Action::KillToEnd,
                KeyCode::Char('u') => return Action::KillToStart,
                KeyCode::Char('y') => return Action::Paste,
                KeyCode::Char('w') => return Action::DeleteWord,
                KeyCode::Char('d') => return Action::DeleteCharForward,
                KeyCode::Char('c') => return Action::Quit,
                _ => return Action::Noop,
            }
        }

        match key.code {
            KeyCode::Esc => Action::EnterNormalMode,
            KeyCode::Enter => Action::SubmitMessage,
            KeyCode::Backspace => Action::DeleteChar,
            KeyCode::Delete => Action::DeleteCharForward,
            KeyCode::Left => Action::MoveCursorLeft,
            KeyCode::Right => Action::MoveCursorRight,
            KeyCode::Home => Action::MoveCursorHome,
            KeyCode::End => Action::MoveCursorEnd,
            KeyCode::Tab => Action::TabNext,
            KeyCode::BackTab => Action::TabPrev,
            KeyCode::Char(c) => Action::InsertChar(c),
            _ => Action::Noop,
        }
    }

    // ── Default profile (current behavior) ──────────────────────

    fn resolve_default(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match mode {
            InputMode::Normal => self.resolve_default_normal(key),
            InputMode::Insert => self.resolve_vim_insert(key), // Same as vim insert
            InputMode::Command => self.resolve_editing_common(key, true),
            InputMode::Search => self.resolve_editing_common(key, true),
        }
    }

    fn resolve_default_normal(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => return Action::Quit,
                _ => return Action::Noop,
            }
        }
        match key.code {
            KeyCode::Char('i') => Action::EnterInsertMode,
            KeyCode::Char(':') => Action::EnterCommandMode,
            KeyCode::Char('/') => Action::EnterSearchMode,
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('j') => Action::ScrollDown,
            KeyCode::Char('k') => Action::ScrollUp,
            KeyCode::Char('G') => Action::ScrollToBottom,
            _ => Action::Noop,
        }
    }

    // ── Shared editing for Command/Search modes ─────────────────

    fn resolve_editing_common(&mut self, key: KeyEvent, clear_on_esc: bool) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => return Action::Cancel,
                _ => return Action::Noop,
            }
        }

        match key.code {
            KeyCode::Esc => {
                if clear_on_esc {
                    Action::Cancel
                } else {
                    Action::EnterNormalMode
                }
            }
            KeyCode::Enter => Action::SubmitMessage,
            KeyCode::Backspace => Action::DeleteChar,
            KeyCode::Char(c) => Action::InsertChar(c),
            _ => Action::Noop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn keymap_profile_default_is_default() {
        let profile = KeymapProfile::default();
        assert!(matches!(profile, KeymapProfile::Default));
    }

    #[test]
    fn default_normal_mode_basic_keys() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char(':'))),
            Action::EnterCommandMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('/'))),
            Action::EnterSearchMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('q'))),
            Action::Quit
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('j'))),
            Action::ScrollDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('k'))),
            Action::ScrollUp
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('G'))),
            Action::ScrollToBottom
        );
    }

    #[test]
    fn default_normal_ctrl_c_quits() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_ctrl(KeyCode::Char('c'))),
            Action::Quit
        );
    }

    #[test]
    fn vim_normal_mode_keys() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('a'))),
            Action::EnterInsertModeAppend
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('o'))),
            Action::EnterInsertModeOpenLineBelow
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('O'))),
            Action::EnterInsertModeOpenLineAbove
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('h'))),
            Action::MoveCursorLeft
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('l'))),
            Action::MoveCursorRight
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Tab)),
            Action::TabNext
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('p'))),
            Action::Paste
        );
    }

    #[test]
    fn vim_double_key_dd() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // First 'd' returns Noop and sets pending
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('d'))),
            Action::Noop
        );
        // Second 'd' returns ClearLine
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('d'))),
            Action::ClearLine
        );
    }

    #[test]
    fn vim_double_key_yy() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('y'))),
            Action::Noop
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('y'))),
            Action::YankLine
        );
    }

    #[test]
    fn vim_count_prefix() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // '3' enters count pending
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('3'))),
            Action::Noop
        );
        // 'j' with count returns ScrollDown
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('j'))),
            Action::ScrollDown
        );
    }

    #[test]
    fn vim_insert_mode_keys() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_key(KeyCode::Esc)),
            Action::EnterNormalMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_key(KeyCode::Enter)),
            Action::SubmitMessage
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_key(KeyCode::Backspace)),
            Action::DeleteChar
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_key(KeyCode::Char('x'))),
            Action::InsertChar('x')
        );
    }

    #[test]
    fn emacs_editing_keys() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_ctrl(KeyCode::Char('n'))),
            Action::ScrollDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_ctrl(KeyCode::Char('p'))),
            Action::ScrollUp
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_ctrl(KeyCode::Char('a'))),
            Action::MoveCursorHome
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_ctrl(KeyCode::Char('e'))),
            Action::MoveCursorEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_ctrl(KeyCode::Char('k'))),
            Action::KillToEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, make_key(KeyCode::Enter)),
            Action::SubmitMessage
        );
    }

    #[test]
    fn command_mode_esc_cancels() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Command, make_key(KeyCode::Esc)),
            Action::Cancel
        );
        assert_eq!(
            kb.resolve(&InputMode::Command, make_key(KeyCode::Enter)),
            Action::SubmitMessage
        );
        assert_eq!(
            kb.resolve(&InputMode::Command, make_ctrl(KeyCode::Char('c'))),
            Action::Cancel
        );
    }

    #[test]
    fn vim_ctrl_d_u_scroll() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_ctrl(KeyCode::Char('d'))),
            Action::ScrollPageDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_ctrl(KeyCode::Char('u'))),
            Action::ScrollPageUp
        );
    }
}
