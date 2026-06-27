//! Layout computation — pure function, no side effects.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 5-zone layout definition
pub struct TerminalLayout {
    pub sidebar: Rect,
    pub chat: Rect,
    pub diff: Rect,
    pub input: Rect,
    pub status: Rect,
}

/// Compute the default layout without transient panels.
pub fn compute_layout(area: Rect) -> TerminalLayout {
    compute_layout_with_state(area, false)
}

/// Compute the terminal-agent layout.
///
/// Wide terminals get a slim context rail. Narrow terminals keep the full width
/// for the conversation. The diff panel is transient and only receives space
/// when there is content to inspect.
pub fn compute_layout_with_state(area: Rect, has_diff: bool) -> TerminalLayout {
    let sidebar_width = if area.width >= 116 { 24 } else { 0 };
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(40)])
        .split(area);

    let diff_height = if has_diff { 9 } else { 0 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(diff_height),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(columns[1]);

    TerminalLayout {
        sidebar: if sidebar_width == 0 {
            Rect::default()
        } else {
            columns[0]
        },
        chat: rows[0],
        diff: if has_diff { rows[1] } else { Rect::default() },
        input: rows[2],
        status: rows[3],
    }
}
