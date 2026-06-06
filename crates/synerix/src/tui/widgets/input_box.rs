//! Multi-mode input box widget (vim/emacs/natural)
//! Polished with rounded borders and theme-aware styling.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, InputMode};
use crate::tui::theme;

/// Render input box showing actual input buffer with cursor
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let p = theme::current_palette();

    let (title, border_color) = match app.mode {
        InputMode::Insert => (" ⌨ Input ", p.chat_assistant),
        InputMode::Normal => (" ○ Normal ", p.chat_user),
        InputMode::Command => (" : Command ", p.chat_system),
        InputMode::Search => (" / Search ", p.chat_tool),
    };

    let border_style = Style::default().fg(border_color);

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(border_style);

    match app.mode {
        InputMode::Insert => {
            if app.input_buffer.is_empty() {
                let hint = "Type your message... (Esc → Normal, Ctrl+C → Quit)";
                let paragraph = Paragraph::new(hint)
                    .block(block)
                    .style(Style::default().fg(p.muted_fg));
                frame.render_widget(paragraph, area);
            } else {
                let cursor_pos = app.input_cursor.min(app.input_buffer.len());
                let before = &app.input_buffer[..cursor_pos];
                let after = &app.input_buffer[cursor_pos..];

                let line = Line::from(vec![
                    Span::styled(before, Style::default().fg(p.foreground)),
                    Span::styled(
                        if after.is_empty() { " " } else { &after[..1] },
                        Style::default()
                            .bg(p.accent)
                            .fg(p.background)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        if after.is_empty() { "" } else { &after[1..] },
                        Style::default().fg(p.foreground),
                    ),
                ]);

                let paragraph = Paragraph::new(line).block(block).wrap(Wrap { trim: false });
                frame.render_widget(paragraph, area);
            }
        }
        InputMode::Normal => {
            let hint = "i=Insert  :=Cmd  /=Search  q=Quit";
            let paragraph = Paragraph::new(hint)
                .block(block)
                .style(Style::default().fg(p.muted_fg));
            frame.render_widget(paragraph, area);
        }
        InputMode::Command => {
            let line = Line::from(vec![
                Span::styled(
                    ":",
                    Style::default()
                        .fg(p.chat_system)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&app.input_buffer, Style::default().fg(p.foreground)),
                Span::styled(" ", Style::default().bg(p.accent).fg(p.background)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
        InputMode::Search => {
            let line = Line::from(vec![
                Span::styled(
                    "/",
                    Style::default()
                        .fg(p.chat_tool)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&app.input_buffer, Style::default().fg(p.foreground)),
                Span::styled(" ", Style::default().bg(p.accent).fg(p.background)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
    }
}
