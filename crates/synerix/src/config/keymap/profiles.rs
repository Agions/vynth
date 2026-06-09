//! Keymap profile definitions — Vim / Emacs / Default key resolution
//!
//! # 优化说明
//! - PendingKey 状态机提取到 `pending.rs`，`resolve_vim_normal`
//!   从 102 行降至 32 行（Ctrl → pending → 普通键 三级流水线）
//! - `resolve_vim_insert` 重命名为 `resolve_insert_common`，
//!   消除「Vim专用」的命名误导（实际被 Default 共享）
//! - 移除 `PendingKey::DoubleKey('g')` 中遗留的 `gg` 注释矛盾

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use super::actions::Action;
use super::pending::PendingKey;
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

    // ── Vim profile ───────────────────────────────────────────────────────

    fn resolve_vim(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match mode {
            InputMode::Normal => self.resolve_vim_normal(key),
            InputMode::Insert => self.resolve_insert_common(key),
            InputMode::Command => self.resolve_editing_common(key, true),
            InputMode::Search => self.resolve_editing_common(key, true),
        }
    }

    /// 三级流水线：Ctrl 组合 → Pending 状态 → 普通单键派发
    fn resolve_vim_normal(&mut self, key: KeyEvent) -> Action {
        // 第一级：Ctrl 组合键
        if let Some(action) = Self::handle_vim_ctrl(key) {
            self.pending_key = None;
            return action;
        }

        // 第二级：Pending 状态解析
        if let Some(pending) = self.pending_key.take() {
            let (action, clear) = pending.try_resolve(key.code);
            if !clear {
                // 计数数字继续累加，不清除 pending
                if let PendingKey::Count(count) = pending {
                    if let KeyCode::Char(d) = key.code {
                        self.pending_key = PendingKey::accumulate_digit(count, d);
                    }
                }
            }
            return action;
        }

        // 第三级：普通单键 + 开启新 pending 序列
        self.resolve_vim_normal_dispatch(key)
    }

    /// Ctrl 组合键 — 仅在 Vim Normal 模式生效
    fn handle_vim_ctrl(key: KeyEvent) -> Option<Action> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('d') => Some(Action::ScrollPageDown),
                KeyCode::Char('u') => Some(Action::ScrollPageUp),
                KeyCode::Char('c') => Some(Action::Quit),
                _ => Some(Action::Noop),
            };
        }
        None
    }

    /// 普通单键派发 + 双键 / 计数前缀的开启
    fn resolve_vim_normal_dispatch(&mut self, key: KeyEvent) -> Action {
        match key.code {
            // Mode transitions
            KeyCode::Char('i') => Action::EnterInsertMode,
            KeyCode::Char('a') => Action::EnterInsertModeAppend,
            KeyCode::Char('A') => Action::EnterInsertModeAppend,
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
            // Cursor
            KeyCode::Char('h') => Action::MoveCursorLeft,
            KeyCode::Char('l') => Action::MoveCursorRight,
            // Tab navigation
            KeyCode::Tab => Action::TabNext,
            KeyCode::BackTab => Action::TabPrev,
            // Paste
            KeyCode::Char('p') => Action::Paste,
            // Double-key / count prefix → try to set pending state
            code => {
                if let Some(pending) = PendingKey::try_set(code) {
                    self.pending_key = Some(pending);
                    Action::Noop
                } else {
                    Action::Noop
                }
            }
        }
    }

    // ── Emacs profile ──────────────────────────────────────────────────────

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
            return match key.code {
                KeyCode::Char('f') => Action::MoveCursorRight,
                KeyCode::Char('b') => Action::MoveCursorLeft,
                KeyCode::Char('v') => Action::ScrollPageUp,
                _ => Action::Noop,
            };
        }

        // Ctrl combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('n') => Action::ScrollDown,
                KeyCode::Char('p') => Action::ScrollUp,
                KeyCode::Char('f') => Action::MoveCursorRight,
                KeyCode::Char('b') => Action::MoveCursorLeft,
                KeyCode::Char('a') => Action::MoveCursorHome,
                KeyCode::Char('e') => Action::MoveCursorEnd,
                KeyCode::Char('k') => Action::KillToEnd,
                KeyCode::Char('u') => Action::KillToStart,
                KeyCode::Char('y') => Action::Paste,
                KeyCode::Char('w') => Action::DeleteWord,
                KeyCode::Char('d') => Action::DeleteCharForward,
                KeyCode::Char('c') => Action::Quit,
                _ => Action::Noop,
            };
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

    // ── Default profile ────────────────────────────────────────────────────

    fn resolve_default(&mut self, mode: &InputMode, key: KeyEvent) -> Action {
        match mode {
            InputMode::Normal => self.resolve_default_normal(key),
            InputMode::Insert => self.resolve_insert_common(key), // 与 Vim Insert 共享
            InputMode::Command => self.resolve_editing_common(key, true),
            InputMode::Search => self.resolve_editing_common(key, true),
        }
    }

    fn resolve_default_normal(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => Action::Quit,
                _ => Action::Noop,
            };
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

    // ── 共享编辑模式（Command / Search / Insert 通用） ─────────────────────

    /// 插入模式通用按键（Vim Insert / Default Insert 相同）
    fn resolve_insert_common(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('w') => Action::DeleteWord,
                KeyCode::Char('a') => Action::MoveCursorHome,
                KeyCode::Char('e') => Action::MoveCursorEnd,
                KeyCode::Char('k') => Action::KillToEnd,
                KeyCode::Char('u') => Action::KillToStart,
                KeyCode::Char('c') => Action::Quit,
                _ => Action::Noop,
            };
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

    /// 命令模式和搜索模式共用按键
    fn resolve_editing_common(&mut self, key: KeyEvent, clear_on_esc: bool) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => Action::Cancel,
                _ => Action::Noop,
            };
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

// ── 测试套件 ─────────────────────────────────────────────────────────────────────

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
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('d'))),
            Action::Noop
        );
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
        assert_eq!(
            kb.resolve(&InputMode::Normal, make_key(KeyCode::Char('3'))),
            Action::Noop
        );
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
