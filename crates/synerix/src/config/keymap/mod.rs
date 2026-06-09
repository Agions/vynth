//! Keymap profiles — maps (InputMode, KeyEvent) → Action

mod actions;
mod pending;
mod profiles;

pub use actions::Action;
pub use profiles::{KeyBindings, KeymapProfile};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::InputMode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
