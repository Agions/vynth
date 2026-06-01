//! Frame renderer — orchestration layer
//!
//! Coordinates layout computation and delegates rendering to each widget.

use crate::app::App;
use crate::tui::layout::{compute_layout, TerminalLayout};
use crate::tui::widgets;
use ratatui::Frame;

/// Draw the entire frame (read-only)
pub fn draw_frame(frame: &mut Frame, app: &App) {
    let layout = compute_layout(frame.area());
    render_all(frame, app, &layout);
}

/// Draw the entire frame and store layout rects in app for mouse hit-testing
pub fn draw_frame_with_layout(frame: &mut Frame, app: &mut App) {
    let layout = compute_layout(frame.area());
    app.layout_state.sidebar_rect = layout.sidebar;
    app.layout_state.chat_rect = layout.chat;
    app.layout_state.diff_rect = layout.diff;
    app.layout_state.input_rect = layout.input;
    app.layout_state.status_rect = layout.status;
    render_all(frame, app, &layout);
}

/// Dispatch rendering to each widget based on computed layout
fn render_all(frame: &mut Frame, app: &App, layout: &TerminalLayout) {
    widgets::sidebar::render(layout.sidebar, frame, app);
    widgets::chat_area::render(layout.chat, frame, app);
    widgets::diff_view::render(layout.diff, frame, app);
    widgets::input_box::render(layout.input, frame, app);
    widgets::status_bar::render_status_bar(frame, app, layout.status);
}
