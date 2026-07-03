//! Layout computation — pure function, no side effects.
//!
//! Single-column focused layout:
//! - **All widths**: full-width chat, no sidebar
//! - Diff panel is transient and only receives space when there is content

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 5-zone layout definition (sidebar always empty in single-column mode)
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

/// Compute a single-column layout with no sidebar.
///
/// Layout (top to bottom):
///   ┌──────────────────────┐
///   │     Chat area        │  ← fills available space
///   ├──────────────────────┤
///   │   Diff (optional)    │  ← 0 rows when no diff content
///   ├──────────────────────┤
///   │   Input box          │  ← 3-4 rows
///   ├──────────────────────┤
///   │   Status bar         │  ← 1 row
///   └──────────────────────┘
pub fn compute_layout_with_state(area: Rect, has_diff: bool) -> TerminalLayout {
    // No sidebar — use full width
    let sidebar = Rect::default();

    // Responsive diff height
    let diff_height = if has_diff {
        if area.height >= 30 {
            12
        } else if area.height >= 20 {
            8
        } else {
            5
        }
    } else {
        0
    };

    // Responsive input height
    let input_height = if area.height >= 15 { 4 } else { 3 };
    let status_height = 1;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(if has_diff { 5 } else { 3 }),
            Constraint::Length(diff_height),
            Constraint::Length(input_height),
            Constraint::Length(status_height),
        ])
        .split(area);

    TerminalLayout {
        sidebar,
        chat: rows[0],
        diff: if has_diff { rows[1] } else { Rect::default() },
        input: rows[rows.len() - 2],
        status: rows[rows.len() - 1],
    }
}
