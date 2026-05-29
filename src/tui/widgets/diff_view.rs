//! Diff view widget — unified and side-by-side diff display

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

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
                DiffLineKind::Add => ("+ ", Color::Green),
                DiffLineKind::Remove => ("- ", Color::Red),
                DiffLineKind::Context => ("  ", Color::Gray),
                DiffLineKind::Header => ("@@ ", Color::Cyan),
            };

            let styled_line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(&line.content, Style::default().fg(color)),
            ]);

            buf.set_line(area.x, area.y + i as u16, &styled_line, area.width);
        }
    }
}
