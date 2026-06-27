//! Multi-mode terminal composer.

use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, InputMode};
use crate::tui::theme;

/// Render input box showing actual input buffer with cursor
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let p = theme::current_palette();
    let cursor_style = if app.status_bar.animation_frame % 4 < 2 {
        Style::default().bg(p.accent).fg(p.background)
    } else {
        Style::default().fg(p.accent)
    };

    let is_slash = app.input_buffer.starts_with('/');
    let (title, prefix, border_color) = match app.mode {
        InputMode::Insert if is_slash => (" command ", "", p.accent),
        InputMode::Insert => (" ask ", "> ", p.chat_assistant),
        InputMode::Normal => (" normal ", "· ", p.chat_user),
        InputMode::Command => (" shell ", ": ", p.chat_system),
        InputMode::Search => (" search ", "/ ", p.chat_tool),
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
        .border_type(ratatui::widgets::BorderType::Plain)
        .border_style(border_style);

    match app.mode {
        InputMode::Insert => {
            if app.input_buffer.is_empty() {
                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(border_color)),
                    Span::styled(" ", cursor_style),
                    Span::styled(
                        "Type a task or / for commands · Tab switches mode",
                        Style::default().fg(p.muted_fg),
                    ),
                ]);
                let paragraph = Paragraph::new(line)
                    .block(block)
                    .style(Style::default().fg(p.muted_fg));
                frame.render_widget(paragraph, area);
                set_input_cursor(area, frame, prefix.len(), 0);
            } else {
                let cursor_pos = app.input_cursor.min(app.input_buffer.len());
                let before = &app.input_buffer[..cursor_pos];
                let after = &app.input_buffer[cursor_pos..];
                let cursor_char_len = after.chars().next().map(char::len_utf8).unwrap_or(0);
                let (cursor, rest) = after.split_at(cursor_char_len);

                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(border_color)),
                    Span::styled(before, Style::default().fg(p.foreground)),
                    Span::styled(
                        if cursor.is_empty() { " " } else { cursor },
                        cursor_style.add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(rest, Style::default().fg(p.foreground)),
                ]);

                let paragraph = Paragraph::new(line).block(block).wrap(Wrap { trim: false });
                frame.render_widget(paragraph, area);
                set_input_cursor(area, frame, prefix.len(), before.chars().count());
            }
        }
        InputMode::Normal => {
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(border_color)),
                Span::styled(
                    "i insert  : command  / commands  tab mode  q quit",
                    Style::default().fg(p.muted_fg),
                ),
            ]);
            let paragraph = Paragraph::new(line)
                .block(block)
                .style(Style::default().fg(p.muted_fg));
            frame.render_widget(paragraph, area);
        }
        InputMode::Command => {
            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(p.chat_system)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&app.input_buffer, Style::default().fg(p.foreground)),
                Span::styled(" ", cursor_style),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
            set_input_cursor(area, frame, prefix.len(), app.input_buffer.chars().count());
        }
        InputMode::Search => {
            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(p.chat_tool)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&app.input_buffer, Style::default().fg(p.foreground)),
                Span::styled(" ", cursor_style),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
            set_input_cursor(area, frame, prefix.len(), app.input_buffer.chars().count());
        }
    }
}

fn set_input_cursor(area: Rect, frame: &mut Frame, prefix_width: usize, input_width: usize) {
    let max_x = area.width.saturating_sub(2) as usize;
    let x = (prefix_width + input_width).min(max_x) as u16;
    frame.set_cursor_position(Position::new(area.x + 1 + x, area.y + 1));
}
