//! Slash command suggestions rendered above the composer.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::slash::menu::menu_matches;
use crate::tui::theme;

pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let matches = menu_matches(&app.input_buffer);
    if matches.is_empty() {
        return;
    }

    let height = (matches.len() as u16 + 2).min(area.y);
    if height < 3 {
        return;
    }

    let width = area.width.min(78);
    let x = area.x + 1;
    let y = area.y.saturating_sub(height);
    let popup = Rect::new(x, y, width.saturating_sub(2), height);
    let p = theme::current_palette();

    let block = Block::default()
        .title(" commands ")
        .title_style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Plain)
        .border_style(Style::default().fg(p.border));

    let active_style = if app.status_bar.animation_frame % 4 < 2 {
        Style::default()
            .fg(p.foreground)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
    };

    let lines: Vec<Line> = matches
        .into_iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let name_style = if idx == app.slash_menu_state.selected {
                active_style
            } else {
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
            };
            Line::from(vec![
                Span::styled(
                    if idx == app.slash_menu_state.selected {
                        "> "
                    } else {
                        "  "
                    },
                    Style::default().fg(p.muted_fg),
                ),
                Span::styled(format!("{:<11}", cmd.name), name_style),
                Span::styled(cmd.desc, Style::default().fg(p.muted_fg)),
            ])
        })
        .collect();

    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
