//! Approval popup widget — shows tool preview and asks for y/n confirmation

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

/// Render an approval popup overlay when approval is pending
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let preview = match &app.pending_approval {
        Some(text) => text,
        None => return,
    };

    // Dimmed background overlay
    let overlay = Rect {
        x: area.width / 6,
        y: area.height / 4,
        width: area.width * 2 / 3,
        height: area.height / 3,
    };

    // Clear area for popup
    frame.render_widget(Clear, overlay);

    // Popup block
    let block = Block::default()
        .title(" 🔍 Tool Approval Required ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));

    // Preview text
    let text = vec![
        Line::from(Span::styled(
            "The following tool requires your approval:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            truncate_preview(preview, 500),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press (y)es to allow · (n)o to deny · (a)lways allow for this session",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let inner = block.inner(overlay);
    frame.render_widget(Clear, inner);
    frame.render_widget(block, overlay);

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));
    frame.render_widget(paragraph, inner);
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        format!(
            "{}…\n\n⚠️ Preview truncated ({} chars shown of {})",
            &text[..max_chars],
            max_chars,
            text.len()
        )
    }
}
