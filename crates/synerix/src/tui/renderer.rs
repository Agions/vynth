//! Frame renderer — orchestration layer
//!
//! Coordinates layout computation and delegates rendering to each widget.
//! Renders each visible widget on every frame.
//!
//! Uses [`RenderContext`] to pass a lightweight snapshot of app state to
//! each widget, keeping render signatures uniform and testable.

use crate::app::{FocusedPanel, App};
use crate::tui::animation::blink_alpha;
use crate::tui::layout::compute_layout_with_state;
use crate::tui::widgets::{self, Widget};
use crate::tui::widgets::primitives::{FocusRing, ShadowLine};
use ratatui::Frame;

/// Draw the entire frame and store layout rects in app for mouse hit-testing.
/// Clears dirty flags for rendered widgets only.
pub fn draw_frame_with_layout(frame: &mut Frame, app: &mut App) {
    let layout = compute_layout_with_state(frame.area(), !app.diff_state.content.is_empty());
    app.layout_state.sidebar_rect = layout.sidebar;
    app.layout_state.chat_rect = layout.chat;
    app.layout_state.diff_rect = layout.diff;
    app.layout_state.input_rect = layout.input;
    app.layout_state.status_rect = layout.status;

    // Build a lightweight render context once per frame.
    let ctx = widgets::primitives::RenderContext::from_app(app);

    // Track which widgets we actually render so we clear only those dirty flags.
    let mut rendered = crate::app::DirtyFlags::empty();

    // ── Sidebar ─────────────────────────────────────────────────────────
    if layout.sidebar.width > 0 {
        widgets::sidebar::render(layout.sidebar, frame, &ctx);
        rendered.insert(crate::app::DirtyFlags::SIDEBAR);
    }

    // ── Chat area ───────────────────────────────────────────────────────
    widgets::chat_area::render(layout.chat, frame, &ctx);
    rendered.insert(crate::app::DirtyFlags::CHAT);

    // ── Diff panel ──────────────────────────────────────────────────────
    if layout.diff.height > 0 {
        widgets::diff_view::render(layout.diff, frame, &ctx);
        rendered.insert(crate::app::DirtyFlags::DIFF);
    }

    // ── Input box ───────────────────────────────────────────────────────
    widgets::input_box::render(layout.input, frame, &ctx);
    rendered.insert(crate::app::DirtyFlags::INPUT);

    // ── Slash menu (overlay on input area) ───────────────────────────────
    widgets::slash_menu::render(layout.input, frame, &ctx);

    // ── Status bar ──────────────────────────────────────────────────────
    widgets::status_bar::render_status_bar(frame, &ctx, layout.status);
    rendered.insert(crate::app::DirtyFlags::STATUS);

    // ── Approval popup (top-most overlay) ────────────────────────────────
    if ctx.approval_pending {
        widgets::approval_popup::render(frame, &ctx, frame.area());
        rendered.insert(crate::app::DirtyFlags::APPROVAL);
    }

    // ── Focus rings (rendered last so they sit on top) ───────────────────
    // Use a subtle pulse animation for the focused panel's focus ring.
    let focus_alpha = if ctx.goal_active || ctx.approval_pending {
        // More prominent pulse when something important is happening
        blink_alpha(ctx.anim_frame, 40)
    } else {
        blink_alpha(ctx.anim_frame, 80)
    };

    if ctx.is_focused(FocusedPanel::Chat) {
        FocusRing::new(FocusedPanel::Chat)
            .with_alpha(focus_alpha)
            .render(layout.chat, frame, &ctx);
    } else if ctx.is_focused(FocusedPanel::Diff) && layout.diff.height > 0 {
        FocusRing::new(FocusedPanel::Diff)
            .with_alpha(focus_alpha)
            .render(layout.diff, frame, &ctx);
    } else if ctx.is_focused(FocusedPanel::Sidebar) && layout.sidebar.width > 0 {
        FocusRing::new(FocusedPanel::Sidebar)
            .with_alpha(focus_alpha)
            .render(layout.sidebar, frame, &ctx);
    } else if ctx.is_focused(FocusedPanel::Input) {
        FocusRing::new(FocusedPanel::Input)
            .with_alpha(focus_alpha)
            .render(layout.input, frame, &ctx);
    }

    // ── Shadow lines for depth ──────────────────────────────────────────
    if layout.input.height > 1 {
        ShadowLine.render(layout.input, frame, &ctx);
    }
    if layout.diff.height > 2 {
        ShadowLine.render(layout.diff, frame, &ctx);
    }

    // Clear dirty flags for rendered widgets; preserve unrendered ones
    // so they get picked up on the next draw pass.
    app.dirty_flags.remove(!rendered);
}
