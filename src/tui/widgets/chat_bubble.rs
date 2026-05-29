//! Chat bubble widget

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

/// Chat message bubble
pub struct ChatBubble<'a> {
    pub role: &'a str,
    pub content: &'a str,
    pub is_streaming: bool,
}

impl<'a> Widget for ChatBubble<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let (prefix, color) = match self.role {
            "user" => ("  You: ", Color::Cyan),
            "assistant" => ("  AI:  ", Color::Green),
            "system" => ("  Sys: ", Color::Yellow),
            _ => ("  ???: ", Color::Gray),
        };

        let cursor = if self.is_streaming { "▌" } else { "" };

        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(self.content),
            Span::styled(cursor, Style::default().fg(Color::Green)),
        ]);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}
