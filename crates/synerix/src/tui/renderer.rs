//! Frame renderer — orchestration layer
//!
//! Coordinates layout computation and delegates rendering to each widget.
//! Renders each visible widget on every frame.

use crate::app::App;
use crate::tui::layout::{compute_layout_with_state, TerminalLayout};
use crate::tui::widgets;
use ratatui::Frame;

/// Draw the entire frame (read-only snapshot; does not clear dirty flags)
#[allow(dead_code)]
pub fn draw_frame(frame: &mut Frame, app: &App) {
    let layout = compute_layout_with_state(frame.area(), !app.diff_state.content.is_empty());
    render_all(frame, app, &layout);
}

/// Draw the entire frame and store layout rects in app for mouse hit-testing.
/// Resets dirty flags after rendering.
#[allow(dead_code)]
pub fn draw_frame_with_layout(frame: &mut Frame, app: &mut App) {
    let layout = compute_layout_with_state(frame.area(), !app.diff_state.content.is_empty());
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
fn render_all(frame: &mut Frame, app: &App, layout: &TerminalLayout) {
    if layout.sidebar.width > 0 {
        widgets::sidebar::render(layout.sidebar, frame, app);
    }
    widgets::chat_area::render(layout.chat, frame, app);
    if layout.diff.height > 0 {
        widgets::diff_view::render(layout.diff, frame, app);
    }
    widgets::input_box::render(layout.input, frame, app);
    widgets::slash_menu::render(layout.input, frame, app);
    widgets::status_bar::render_status_bar(frame, app, layout.status);
    if app.pending_approval.is_some() {
        widgets::approval_popup::render(frame, app, frame.area());
    }
}
