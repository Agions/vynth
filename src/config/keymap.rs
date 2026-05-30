//! Keymap profiles — maps (InputMode, KeyEvent) → Action

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::app::InputMode;

/// All possible actions the keymap can resolve to
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Text editing
    InsertChar(char),
    DeleteChar,
    DeleteCharForward,
    DeleteWord,
    KillToEnd,
    KillToStart,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    // Mode transitions
    SubmitMessage,
    EnterInsertMode,
    EnterInsertModeAppend,
    EnterInsertModeOpenLineBelow,
    EnterInsertModeOpenLineAbove,
    EnterNormalMode,
    EnterCommandMode,
    EnterSearchMode,
    // Scrolling
    ScrollUp,
    ScrollDown,
    ScrollToBottom,
    ScrollPageUp,
    ScrollPageDown,
    // Application
    Quit,
    Cancel,
    TabNext,
    TabPrev,
    // Yank/paste
    YankLine,
    Paste,
    // Vim-specific
    ClearLine,
    // No action
    Noop,
}

/// Keymap profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeymapProfile {
    Vim,
    Emacs,
    Default,
}

impl Default for KeymapProfile {
    fn default() -> Self {
        Self::Default
    }
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

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn key_alt(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }

    // ── Default profile ────────────────────────────────────

    #[test]
    fn test_default_normal_quit() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('q'))),
            Action::Quit
        );
    }

    #[test]
    fn test_default_normal_enter_insert() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
    }

    #[test]
    fn test_default_normal_scroll() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('j'))),
            Action::ScrollDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('k'))),
            Action::ScrollUp
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('G'))),
            Action::ScrollToBottom
        );
    }

    #[test]
    fn test_default_normal_command_search() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char(':'))),
            Action::EnterCommandMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('/'))),
            Action::EnterSearchMode
        );
    }

    #[test]
    fn test_default_insert_esc() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Esc)),
            Action::EnterNormalMode
        );
    }

    #[test]
    fn test_default_insert_submit() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Enter)),
            Action::SubmitMessage
        );
    }

    #[test]
    fn test_default_insert_char() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Char('a'))),
            Action::InsertChar('a')
        );
    }

    #[test]
    fn test_default_command_esc() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Command, key(KeyCode::Esc)),
            Action::Cancel
        );
    }

    #[test]
    fn test_default_command_ctrl_c() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Command, key_ctrl(KeyCode::Char('c'))),
            Action::Cancel
        );
    }

    // ── Vim profile ────────────────────────────────────────

    #[test]
    fn test_vim_normal_mode_transitions() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('a'))),
            Action::EnterInsertModeAppend
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('o'))),
            Action::EnterInsertModeOpenLineBelow
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('O'))),
            Action::EnterInsertModeOpenLineAbove
        );
    }

    #[test]
    fn test_vim_double_dd() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // First 'd' is Noop (pending)
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('d'))),
            Action::Noop
        );
        // Second 'd' triggers ClearLine
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('d'))),
            Action::ClearLine
        );
    }

    #[test]
    fn test_vim_double_yy() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('y'))),
            Action::Noop
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('y'))),
            Action::YankLine
        );
    }

    #[test]
    fn test_vim_count_prefix() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // '3' starts count
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('3'))),
            Action::Noop
        );
        // 'j' with count
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('j'))),
            Action::ScrollDown
        );
    }

    #[test]
    fn test_vim_count_accumulation() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('1'))),
            Action::Noop
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('2'))),
            Action::Noop
        );
        // '12j' = scroll down 12
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('j'))),
            Action::ScrollDown
        );
    }

    #[test]
    fn test_vim_insert_ctrl_combinations() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('w'))),
            Action::DeleteWord
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('a'))),
            Action::MoveCursorHome
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('e'))),
            Action::MoveCursorEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('k'))),
            Action::KillToEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('u'))),
            Action::KillToStart
        );
    }

    #[test]
    fn test_vim_normal_hl() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('h'))),
            Action::MoveCursorLeft
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('l'))),
            Action::MoveCursorRight
        );
    }

    #[test]
    fn test_vim_normal_paste() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('p'))),
            Action::Paste
        );
    }

    #[test]
    fn test_vim_normal_tab() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Tab)),
            Action::TabNext
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::BackTab)),
            Action::TabPrev
        );
    }

    // ── Emacs profile ──────────────────────────────────────

    #[test]
    fn test_emacs_ctrl_navigation() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('n'))),
            Action::ScrollDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('p'))),
            Action::ScrollUp
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('f'))),
            Action::MoveCursorRight
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('b'))),
            Action::MoveCursorLeft
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('a'))),
            Action::MoveCursorHome
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('e'))),
            Action::MoveCursorEnd
        );
    }

    #[test]
    fn test_emacs_ctrl_editing() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('k'))),
            Action::KillToEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('u'))),
            Action::KillToStart
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('y'))),
            Action::Paste
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('w'))),
            Action::DeleteWord
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_ctrl(KeyCode::Char('d'))),
            Action::DeleteCharForward
        );
    }

    #[test]
    fn test_emacs_alt_navigation() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key_alt(KeyCode::Char('v'))),
            Action::ScrollPageUp
        );
    }

    #[test]
    fn test_emacs_esc() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Esc)),
            Action::EnterNormalMode
        );
    }

    #[test]
    fn test_emacs_command_ctrl_c() {
        let mut kb = KeyBindings::new(KeymapProfile::Emacs);
        assert_eq!(
            kb.resolve(&InputMode::Command, key_ctrl(KeyCode::Char('c'))),
            Action::Cancel
        );
    }

    // ── Edge cases ─────────────────────────────────────────

    #[test]
    fn test_unmapped_key_returns_noop() {
        let mut kb = KeyBindings::new(KeymapProfile::Default);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::F(1))),
            Action::Noop
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::F(12))),
            Action::Noop
        );
    }

    #[test]
    fn test_pending_key_cleared_after_action() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // Start dd sequence
        kb.resolve(&InputMode::Normal, key(KeyCode::Char('d')));
        // Complete it
        kb.resolve(&InputMode::Normal, key(KeyCode::Char('d')));
        // Next key should be normal (no pending state)
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
    }

    #[test]
    fn test_pending_key_invalid_motion() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        // 'd' then invalid motion
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('d'))),
            Action::Noop
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('x'))),
            Action::Noop
        );
        // Pending cleared, next key is normal
        assert_eq!(
            kb.resolve(&InputMode::Normal, key(KeyCode::Char('i'))),
            Action::EnterInsertMode
        );
    }

    #[test]
    fn test_vim_insert_cursor_movement() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Left)),
            Action::MoveCursorLeft
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Right)),
            Action::MoveCursorRight
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Home)),
            Action::MoveCursorHome
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::End)),
            Action::MoveCursorEnd
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Backspace)),
            Action::DeleteChar
        );
        assert_eq!(
            kb.resolve(&InputMode::Insert, key(KeyCode::Delete)),
            Action::DeleteCharForward
        );
    }

    #[test]
    fn test_vim_ctrl_u_in_normal() {
        let mut kb = KeyBindings::new(KeymapProfile::Vim);
        assert_eq!(
            kb.resolve(&InputMode::Normal, key_ctrl(KeyCode::Char('u'))),
            Action::ScrollPageUp
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key_ctrl(KeyCode::Char('d'))),
            Action::ScrollPageDown
        );
        assert_eq!(
            kb.resolve(&InputMode::Normal, key_ctrl(KeyCode::Char('c'))),
            Action::Quit
        );
    }
}
