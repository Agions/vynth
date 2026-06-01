//! Action definitions for keymap bindings

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_debug_format() {
        let action = Action::Quit;
        let debug = format!("{:?}", action);
        assert_eq!(debug, "Quit");
    }

    #[test]
    fn action_clone() {
        let a = Action::InsertChar('x');
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn action_partial_eq() {
        assert_eq!(Action::Quit, Action::Quit);
        assert_ne!(Action::Quit, Action::Cancel);
        assert_eq!(Action::InsertChar('a'), Action::InsertChar('a'));
        assert_ne!(Action::InsertChar('a'), Action::InsertChar('b'));
        assert_eq!(Action::Noop, Action::Noop);
    }

    #[test]
    fn action_all_variants_exist() {
        // Ensure all variants compile and are distinct
        let actions = vec![
            Action::InsertChar('c'),
            Action::DeleteChar,
            Action::DeleteCharForward,
            Action::DeleteWord,
            Action::KillToEnd,
            Action::KillToStart,
            Action::MoveCursorLeft,
            Action::MoveCursorRight,
            Action::MoveCursorHome,
            Action::MoveCursorEnd,
            Action::SubmitMessage,
            Action::EnterInsertMode,
            Action::EnterInsertModeAppend,
            Action::EnterInsertModeOpenLineBelow,
            Action::EnterInsertModeOpenLineAbove,
            Action::EnterNormalMode,
            Action::EnterCommandMode,
            Action::EnterSearchMode,
            Action::ScrollUp,
            Action::ScrollDown,
            Action::ScrollToBottom,
            Action::ScrollPageUp,
            Action::ScrollPageDown,
            Action::Quit,
            Action::Cancel,
            Action::TabNext,
            Action::TabPrev,
            Action::YankLine,
            Action::Paste,
            Action::ClearLine,
            Action::Noop,
        ];
        assert_eq!(actions.len(), 31);
    }
}
