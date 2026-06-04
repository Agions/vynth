//! Chat bubble widget with polished visual styling
// TODO: Chat bubble widget — not yet wired
#![allow(dead_code)]

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::tui::theme;

/// Chat message bubble with rounded corners, role header, and proper padding.
///
/// Renders messages in a visually distinct bubble format:
/// ```text
/// ╭─ You (12:34) ─────────────────────────╮
/// │ Hello, can you help me with this?      │
/// ╰────────────────────────────────────────╯
/// ```
pub struct ChatBubble<'a> {
    pub role: &'a str,
    pub content: &'a str,
    pub is_streaming: bool,
    /// Optional timestamp string (e.g. "12:34")
    pub timestamp: Option<&'a str>,
}

impl<'a> ChatBubble<'a> {
    /// Create a new ChatBubble (backward-compatible constructor)
    pub fn new(role: &'a str, content: &'a str, is_streaming: bool) -> Self {
        Self {
            role,
            content,
            is_streaming,
            timestamp: None,
        }
    }

    /// Set an optional timestamp for the bubble header
    pub fn with_timestamp(mut self, timestamp: &'a str) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Resolve display name and accent color for a role
    fn role_style(&self) -> (&'static str, Color) {
        match self.role {
            "user" => ("You", Color::Cyan),
            "assistant" => ("AI", Color::Green),
            "system" => ("Sys", Color::Yellow),
            _ => ("???", Color::Gray),
        }
    }

    /// Background color used to fill the bubble interior
    fn bg_color(&self) -> Color {
        match self.role {
            "user" => Color::Rgb(20, 40, 55),      // dark teal
            "assistant" => Color::Rgb(20, 45, 20), // dark green
            "system" => Color::Rgb(50, 45, 15),    // dark yellow
            _ => Color::Rgb(30, 30, 30),           // neutral dark gray
        }
    }

    /// Wrap content lines to fit within `inner_width` columns.
    /// Returns a Vec of trimmed line strings.
    fn wrap_lines(&self, inner_width: u16) -> Vec<String> {
        let max_w = inner_width as usize;
        let mut lines = Vec::new();

        for raw_line in self.content.split('\n') {
            if raw_line.is_empty() {
                lines.push(String::new());
                continue;
            }
            // Simple word-wrap
            let mut current = String::new();
            for word in raw_line.split_whitespace() {
                if current.is_empty() {
                    if word.len() > max_w {
                        // Hard-break a single long word
                        for chunk in word.as_bytes().chunks(max_w) {
                            lines.push(String::from_utf8_lossy(chunk).into_owned());
                        }
                    } else {
                        current.push_str(word);
                    }
                } else if current.len() + 1 + word.len() <= max_w {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    lines.push(current);
                    if word.len() > max_w {
                        for chunk in word.as_bytes().chunks(max_w) {
                            lines.push(String::from_utf8_lossy(chunk).into_owned());
                        }
                        current = String::new();
                    } else {
                        current = String::from(word);
                    }
                }
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }

        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }
}

impl<'a> Widget for ChatBubble<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 4 {
            // Minimum: top border + 1 content line + bottom border
            return;
        }

        let (role_name, accent) = self.role_style();
        let bg = self.bg_color();
        let _role_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
        let border_style = Style::default().fg(theme::COLOR_DARK_GRAY).bg(bg);
        let content_style = Style::default().fg(theme::COLOR_WHITE).bg(bg);

        let cursor = if self.is_streaming { "▌" } else { "" };

        let inner_width = area.width.saturating_sub(4) as usize; // "│ " prefix + " │" suffix
        if inner_width == 0 {
            return;
        }

        // Build the header label: "─ Role (HH:MM) ─"
        let label = match self.timestamp {
            Some(ts) => format!(" {} ({}) ", role_name, ts),
            None => format!(" {} ", role_name),
        };

        // ── Top border ──────────────────────────────────────────────────
        let top = build_horizontal_border('╭', '─', '╮', &label, accent, area.width);
        buf.set_line(area.x, area.y, &top, area.width);

        // ── Content lines ───────────────────────────────────────────────
        let wrapped = self.wrap_lines(inner_width as u16);
        let content_rows = (area.height.saturating_sub(2)) as usize; // rows available (top+bottom taken)

        for (i, line_text) in wrapped.iter().take(content_rows).enumerate() {
            let row = area.y + 1 + i as u16;
            if row >= area.y + area.height - 1 {
                break;
            }

            // Left border: "│ "
            buf.set_string(area.x, row, "│ ", border_style);

            // Content padded to inner width
            let mut display = line_text.clone();
            // Append streaming cursor on last visible content line
            if self.is_streaming && i == wrapped.len().saturating_sub(1) {
                display.push_str(cursor);
            }
            let pad = inner_width.saturating_sub(display.chars().count());
            let padded = format!("{}{}", display, " ".repeat(pad));
            buf.set_string(area.x + 2, row, &padded, content_style);

            // Right border: " │"
            let right_x = area.x + 2 + inner_width as u16;
            buf.set_string(right_x, row, " │", border_style);
        }

        // Fill any remaining empty rows inside the bubble
        for i in wrapped.len()..content_rows {
            let row = area.y + 1 + i as u16;
            if row >= area.y + area.height - 1 {
                break;
            }
            buf.set_string(area.x, row, "│ ", border_style);
            let pad_str = " ".repeat(inner_width);
            buf.set_string(area.x + 2, row, &pad_str, content_style);
            let right_x = area.x + 2 + inner_width as u16;
            buf.set_string(right_x, row, " │", border_style);
        }

        // ── Bottom border ───────────────────────────────────────────────
        let bottom_row = area.y + area.height - 1;
        let bottom = build_horizontal_border('╰', '─', '╯', "", accent, area.width);
        buf.set_line(area.x, bottom_row, &bottom, area.width);
    }
}

/// Build a horizontal border line with a label in the middle.
///
/// E.g.: `╭─ You (12:34) ────────────────╮`
fn build_horizontal_border<'b>(
    left: char,
    fill: char,
    right: char,
    label: &str,
    label_color: Color,
    width: u16,
) -> Line<'b> {
    let w = width as usize;
    if w < 2 {
        return Line::from(Span::raw(left.to_string()));
    }

    let label_str = if !label.is_empty() && w > 4 {
        // Ensure we have the dash prefix before label
        format!("{}─{}", left, label)
    } else {
        left.to_string()
    };

    let label_len = label_str.chars().count();
    let remaining = w.saturating_sub(label_len + 1); // 1 for right corner
    let fill_str: String = fill.to_string().repeat(remaining);
    let right_str = right.to_string();

    let border_color = theme::COLOR_DARK_GRAY;

    if label.is_empty() {
        // Plain border, no label
        let plain = format!(
            "{}{}{}",
            left,
            fill.to_string().repeat(w.saturating_sub(2)),
            right
        );
        Line::from(Span::styled(plain, theme::muted_style()))
    } else {
        Line::from(vec![
            Span::styled(
                label_str,
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(fill_str, Style::default().fg(border_color)),
            Span::styled(right_str, Style::default().fg(border_color)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn test_empty_area() {
        let bubble = ChatBubble::new("user", "hello", false);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_minimum_size() {
        let bubble = ChatBubble::new("user", "hello", false);
        let area = Rect::new(0, 0, 3, 2); // too small
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_basic_render() {
        let bubble = ChatBubble::new("user", "Hello, world!", false);
        let area = Rect::new(0, 0, 50, 5);
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Top-left corner should be ╭
        assert_eq!(buf[(0, 0)].symbol(), "╭");
        // Bottom-left corner should be ╰
        assert_eq!(buf[(0, 4)].symbol(), "╰");
    }

    #[test]
    fn test_word_wrap() {
        let bubble = ChatBubble {
            role: "user",
            content: "This is a long message that should be wrapped across multiple lines",
            is_streaming: false,
            timestamp: None,
        };
        let lines = bubble.wrap_lines(20);
        assert!(lines.len() > 1, "Content should wrap to multiple lines");
        for line in &lines {
            assert!(line.len() <= 20, "Line exceeds inner width: '{}'", line);
        }
    }

    #[test]
    fn test_timestamp_in_header() {
        let bubble = ChatBubble::new("assistant", "Sure!", false).with_timestamp("14:30");
        let area = Rect::new(0, 0, 50, 4);
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Header should include "AI (14:30)"
        let header_line: String = (0..50).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        assert!(
            header_line.contains("AI (14:30)"),
            "Header missing timestamp: {}",
            header_line
        );
    }

    #[test]
    fn test_multiline_content() {
        let bubble = ChatBubble {
            role: "user",
            content: "Line one\nLine two\nLine three",
            is_streaming: false,
            timestamp: None,
        };
        let area = Rect::new(0, 0, 50, 7); // 3 content rows + 2 borders + 2 spare
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Second content row (y=2) should contain "Line two"
        let row2: String = (2..48).map(|x| buf[(x, 2)].symbol().to_string()).collect();
        assert!(
            row2.contains("Line two"),
            "Multiline not rendered: {}",
            row2
        );
    }

    #[test]
    fn test_streaming_cursor() {
        let bubble = ChatBubble::new("assistant", "typing", true);
        let area = Rect::new(0, 0, 50, 4);
        let mut buf = Buffer::empty(area);
        bubble.render(area, &mut buf);
        // Content row (y=1) should contain the streaming cursor ▌
        let row1: String = (2..48).map(|x| buf[(x, 1)].symbol().to_string()).collect();
        assert!(row1.contains("▌"), "Streaming cursor missing: {}", row1);
    }
}
