//! Diff view widget — unified and side-by-side diff display
// TODO: Diff view widget — not yet wired
#![allow(dead_code)]

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::tui::theme;

pub struct DiffView<'a> {
    pub lines: &'a [DiffLine],
}

pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

pub enum DiffLineKind {
    Add,
    Remove,
    Context,
    Header,
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, line) in self.lines.iter().take(area.height as usize).enumerate() {
            let (prefix, color) = match line.kind {
                DiffLineKind::Add => ("+ ", theme::COLOR_GREEN),
                DiffLineKind::Remove => ("- ", theme::COLOR_RED),
                DiffLineKind::Context => ("  ", theme::COLOR_GRAY),
                DiffLineKind::Header => ("@@ ", theme::COLOR_CYAN),
            };

            let styled_line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(&line.content, Style::default().fg(color)),
            ]);

            buf.set_line(area.x, area.y + i as u16, &styled_line, area.width);
        }
    }
}

/// Render diff preview panel
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let block = Block::default()
        .title(" Diff Preview ")
        .borders(Borders::ALL)
        .border_style(if app.focused_panel == crate::app::FocusedPanel::Diff {
            Style::default().fg(theme::COLOR_CYAN)
        } else {
            theme::muted_style()
        });

    if app.diff_state.content.is_empty() {
        let paragraph = Paragraph::new("  (no pending changes)")
            .block(block)
            .style(theme::muted_style());
        frame.render_widget(paragraph, area);
    } else {
        // Compute available inner width for side-by-side column sizing
        let inner_width = area.width.saturating_sub(3) as usize; // border + gutter
        let col_width = inner_width / 2;

        let diff_text = crate::tui::diff_renderer::render_diff(
            &app.diff_state.content,
            crate::tui::diff_renderer::DiffViewMode::SideBySide,
            col_width,
        );

        // Apply scroll offset
        let paragraph = Paragraph::new(diff_text)
            .block(block)
            .scroll((app.diff_state.scroll_offset as u16, 0));
        frame.render_widget(paragraph, area);
    }
}
