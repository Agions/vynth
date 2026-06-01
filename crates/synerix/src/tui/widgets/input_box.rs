//! Multi-mode input box widget (vim/emacs/natural)

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, InputMode};

pub struct InputBox {
    pub buffer: String,
    pub cursor_pos: usize,
    pub mode: InputBoxMode,
}

pub enum InputBoxMode {
    Insert,
    Normal,
    Command,
    Search,
}

impl Default for InputBox {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBox {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
            mode: InputBoxMode::Insert,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.buffer.remove(self.cursor_pos);
        }
    }

    pub fn submit(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}

/// Render input box showing actual input buffer with cursor
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let (title, border_color) = match app.mode {
        InputMode::Insert => (" Input ", Color::Green),
        InputMode::Normal => (" Normal ", Color::Blue),
        InputMode::Command => (" Command ", Color::Yellow),
        InputMode::Search => (" Search ", Color::Magenta),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    match app.mode {
        InputMode::Insert => {
            if app.input_buffer.is_empty() {
                let hint = "Type your message... (Esc → Normal, Ctrl+C → Quit)";
                let paragraph = Paragraph::new(hint)
                    .block(block)
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, area);
            } else {
                // Show actual input text with cursor
                let cursor_pos = app.input_cursor.min(app.input_buffer.len());
                let before = &app.input_buffer[..cursor_pos];
                let after = &app.input_buffer[cursor_pos..];

                let line = Line::from(vec![
                    Span::styled(before, Style::default().fg(Color::White)),
                    Span::styled(
                        if after.is_empty() { " " } else { &after[..1] },
                        Style::default().bg(Color::White).fg(Color::Black),
                    ),
                    Span::raw(if after.is_empty() { "" } else { &after[1..] }),
                ]);

                let paragraph = Paragraph::new(line).block(block).wrap(Wrap { trim: false });
                frame.render_widget(paragraph, area);
            }
        }
        InputMode::Normal => {
            let hint = "i=Insert, :=Cmd, /=Search, q=Quit";
            let paragraph = Paragraph::new(hint)
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(paragraph, area);
        }
        InputMode::Command => {
            let line = Line::from(vec![
                Span::styled(":", Style::default().fg(Color::Yellow)),
                Span::raw(&app.input_buffer),
                Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
        InputMode::Search => {
            let line = Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Magenta)),
                Span::raw(&app.input_buffer),
                Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
    }
}
