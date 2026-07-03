//! Diff view widget — renders the diff preview panel with rounded borders and scroll offset.
//!
//! Features Nerd Font title, padded code lines, and bottom shadow.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::theme;
use crate::tui::widgets::primitives::RenderContext;

/// Shadow line character — Unicode lower half block.
const SHADOW_CHAR: &str = "▄";

/// Render diff preview panel with rounded borders and scroll offset.
pub fn render(area: Rect, frame: &mut Frame, ctx: &RenderContext) {
    let p = ctx.palette;
    let is_focused = ctx.is_focused(crate::app::FocusedPanel::Diff);

    let border_style = if is_focused {
        Style::default().fg(p.border_focus).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.border)
    };

    let block = Block::default()
        .title("[ Diff ]")
        .title_style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(border_style);

    if ctx.diff_content.is_empty() {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "  ✓ No pending changes",
            Style::default().fg(p.success),
        )))
        .block(block);
        frame.render_widget(paragraph, area);
    } else {
        let inner_width = area.width.saturating_sub(3) as usize;
        let col_width = inner_width / 2;

        let diff_text = crate::tui::diff_renderer::render_diff(
            ctx.diff_content,
            crate::tui::diff_renderer::DiffViewMode::SideBySide,
            col_width,
            &p,
        );

        let paragraph = Paragraph::new(diff_text)
            .block(block)
            .scroll((ctx.diff_scroll as u16, 0));
        frame.render_widget(paragraph, area);
    }

    // ── Shadow line ────────────────────────────────────────────────────
    if area.height > 2 {
        let shadow_y = area.y + area.height - 2;
        let shadow_x = area.x + 1;
        let shadow_w = area.width - 2;
        if shadow_w > 0 && shadow_y < area.y + area.height - 1 {
            let shadow_rect = Rect::new(shadow_x, shadow_y, shadow_w, 1);
            let shadow_line = Line::from(Span::styled(
                SHADOW_CHAR.repeat(shadow_w as usize),
                Style::default().fg(p.shadow_color),
            ));
            frame.render_widget(Paragraph::new(shadow_line), shadow_rect);
        }
    }
}
