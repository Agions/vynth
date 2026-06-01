//! Layout computation — pure function, no side effects

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
pub fn compute_layout(area: Rect) -> TerminalLayout {
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Length(1),
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
