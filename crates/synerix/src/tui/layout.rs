//! Layout computation — pure function, no side effects
//! Enhanced with better proportions and visual balance.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 5-zone layout definition
pub struct TerminalLayout {
    pub sidebar: Rect,
    pub chat: Rect,
    pub diff: Rect,
    pub input: Rect,
    pub status: Rect,
}

/// Compute the standard 5-zone layout from frame area
/// with refined proportions for a modern terminal UI.
pub fn compute_layout(area: Rect) -> TerminalLayout {
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(60)])
        .split(area);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),          // Chat — takes remaining space
            Constraint::Length(10),       // Diff preview — compact
            Constraint::Length(3),        // Input bar
            Constraint::Length(1),        // Status bar
        ])
        .split(h_chunks[1]);

    TerminalLayout {
        sidebar: h_chunks[0],
        chat: v_chunks[0],
        diff: v_chunks[1],
        input: v_chunks[2],
        status: v_chunks[3],
    }
}
