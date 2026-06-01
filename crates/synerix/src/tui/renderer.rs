//! Frame renderer — orchestration layer
//!
//! Coordinates layout computation and delegates rendering to each widget.
//! Uses dirty-flag differential rendering to skip widgets that haven't changed.

use crate::app::App;
use crate::tui::layout::{compute_layout, TerminalLayout};
use crate::tui::widgets;
use ratatui::Frame;

/// Draw the entire frame (read-only snapshot; does not clear dirty flags)
pub fn draw_frame(frame: &mut Frame, app: &App) {
    let layout = compute_layout(frame.area());
    render_all(frame, app, &layout);
}

/// Draw the entire frame and store layout rects in app for mouse hit-testing.
/// Resets dirty flags after rendering clean widgets (they are now up to date).
pub fn draw_frame_with_layout(frame: &mut Frame, app: &mut App) {
    let layout = compute_layout(frame.area());
    app.layout_state.sidebar_rect = layout.sidebar;
    app.layout_state.chat_rect = layout.chat;
    app.layout_state.diff_rect = layout.diff;
    app.layout_state.input_rect = layout.input;
    app.layout_state.status_rect = layout.status;
    render_all(frame, app, &layout);
    // Reset all dirty flags after rendering is complete
    app.dirty_flags = crate::app::DirtyFlags::default();
}

/// Dispatch rendering to each widget based on computed layout.
/// Skips widgets whose dirty flag is not set.
fn render_all(frame: &mut Frame, app: &App, layout: &TerminalLayout) {
    if app.dirty_flags.sidebar {
        widgets::sidebar::render(layout.sidebar, frame, app);
    }
    if app.dirty_flags.chat {
        widgets::chat_area::render(layout.chat, frame, app);
    }
    if app.dirty_flags.diff {
        widgets::diff_view::render(layout.diff, frame, app);
    }
    if app.dirty_flags.input {
        widgets::input_box::render(layout.input, frame, app);
    }
    if app.dirty_flags.status {
        widgets::status_bar::render_status_bar(frame, app, layout.status);
    }
}
