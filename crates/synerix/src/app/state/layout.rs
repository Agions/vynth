//! Layout rects stored from the last render pass for mouse hit-testing.

use ratatui::layout::Rect;

/// Which panel currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    Chat,
    Diff,
    Sidebar,
    #[default]
    Input,
}

/// Layout rects stored from the last render pass for mouse hit-testing
#[derive(Debug, Clone, Default)]
pub struct LayoutState {
    pub sidebar_rect: Rect,
    pub chat_rect: Rect,
    pub diff_rect: Rect,
    pub input_rect: Rect,
    pub status_rect: Rect,
}
