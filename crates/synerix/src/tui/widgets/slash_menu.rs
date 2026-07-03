//! Slash command suggestions rendered above the composer.
//!
//! Rounded border, highlight_bg for selection, Nerd Font arrow indicator.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::slash::menu::menu_matches;
use crate::tui::widgets::primitives::RenderContext;

/// Selected-item indicator (replaces Nerd Font arrow).
const ARROW_ACTIVE: &str = "> ";
const ARROW_INACTIVE: &str = "  ";

/// Render slash command popup above the input box.
pub fn render(area: Rect, frame: &mut Frame, ctx: &RenderContext) {
    let matches = menu_matches(&ctx.input_buffer);
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
    let p = ctx.palette;

    let block = Block::default()
        .title(" commands ")
        .title_style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(crate::tui::theme::BORDER_TYPE)
        .border_style(Style::default().fg(p.border));

    let active_style = if ctx.anim_frame % 4 < 2 {
        Style::default()
            .fg(p.foreground)
            .bg(p.highlight_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(p.accent)
            .bg(p.highlight_bg)
            .add_modifier(Modifier::BOLD)
    };

    let lines: Vec<Line> = matches
        .into_iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let is_selected = idx == ctx.slash_selected;
            let name_style = if is_selected {
                active_style
            } else {
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
            };
            Line::from(vec![
                Span::styled(
                    if is_selected {
                        ARROW_ACTIVE
                    } else {
                        ARROW_INACTIVE
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
