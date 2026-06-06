//! Diff view widget — renders the diff preview panel with rounded borders and scroll offset

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::tui::theme;

/// Render diff preview panel with rounded borders and scroll offset
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let p = theme::current_palette();
    let is_focused = app.focused_panel == crate::app::FocusedPanel::Diff;

    let border_style = if is_focused {
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.border)
    };

    let block = Block::default()
        .title(" 📝 Diff Preview ")
        .title_style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(border_style);

    if app.diff_state.content.is_empty() {
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
            &app.diff_state.content,
            crate::tui::diff_renderer::DiffViewMode::SideBySide,
            col_width,
        );

        let paragraph = Paragraph::new(diff_text)
            .block(block)
            .scroll((app.diff_state.scroll_offset as u16, 0));
        frame.render_widget(paragraph, area);
    }
}
