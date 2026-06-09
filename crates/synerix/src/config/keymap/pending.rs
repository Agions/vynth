//! 按键状态机 — 管理多键序列（双键、计数前缀）
//!
//! # 优化说明
//! 从 profiles.rs 提取 PendingKey 状态机，封装多键序列的
//! 全部状态转换逻辑。外部只需调用 `try_set`、`try_resolve`，
//! 无需关心内部状态表示。

use crossterm::event::KeyCode;

use super::actions::Action;

/// 待处理的中间按键状态
#[derive(Debug, Clone)]
pub enum PendingKey {
    /// 计数前缀（如 `3` 等待移动命令）
    Count(u32),
    /// 双键序列的第一个字符（如 `d` 等待第二个 `d`）
    DoubleKey(char),
}

impl PendingKey {
    /// 尝试将普通按键解析为 pending 状态。如果按键触发
    /// 双键或计数前缀，则设置 pending 并返回 Noop。
    pub fn try_set(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Char('d') => Some(PendingKey::DoubleKey('d')),
            KeyCode::Char('y') => Some(PendingKey::DoubleKey('y')),
            KeyCode::Char('g') => Some(PendingKey::DoubleKey('g')),
            KeyCode::Char(d) if d.is_ascii_digit() && d != '0' => {
                Some(PendingKey::Count(d as u32 - '0' as u32))
            }
            _ => None,
        }
    }

    /// 解析 pending 状态下的按键，返回对应的 Action
    /// 和「是否清除 pending 状态」的标记。
    pub fn try_resolve(&self, key: KeyCode) -> (Action, /* clear_pending */ bool) {
        match self {
            PendingKey::Count(count) => Self::resolve_count_key(*count, key),
            PendingKey::DoubleKey(first) => Self::resolve_double_key(*first, key),
        }
    }

    /// 累加更多计数数字，返回新的 pending（可能 None 表示不清除 pending）
    pub fn accumulate_digit(current: u32, digit_byte: char) -> Option<PendingKey> {
        let new_count = current * 10 + (digit_byte as u32 - '0' as u32);
        Some(PendingKey::Count(new_count))
    }

    // ── 内部解析 ──────────────────────────────────────────────────────────

    fn resolve_count_key(_count: u32, key: KeyCode) -> (Action, bool) {
        match key {
            KeyCode::Char('j') => (Action::ScrollDown, true),
            KeyCode::Char('k') => (Action::ScrollUp, true),
            KeyCode::Char('G') => (Action::ScrollToBottom, true),
            KeyCode::Char('g') => (Action::ScrollToBottom, true),
            KeyCode::Char(d) if d.is_ascii_digit() => {
                // digits keep accumulating, don't clear pending
                (Action::Noop, false)
            }
            _ => (Action::Noop, true), // invalid motion → discard
        }
    }

    fn resolve_double_key(first: char, key: KeyCode) -> (Action, bool) {
        match (first, key) {
            ('d', KeyCode::Char('d')) => (Action::ClearLine, true),
            ('y', KeyCode::Char('y')) => (Action::YankLine, true),
            ('g', KeyCode::Char('g')) => (Action::ScrollToBottom, true),
            _ => (Action::Noop, true),
        }
    }
}
