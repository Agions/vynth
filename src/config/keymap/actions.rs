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
