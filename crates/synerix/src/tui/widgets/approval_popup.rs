//! Approval popup widget — shows tool preview and asks for y/n confirmation.
//!
//! Fully palette-driven with dim overlay and refined typography.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::widgets::primitives::RenderContext;

/// Render an approval popup overlay when approval is pending.
pub fn render(frame: &mut Frame, ctx: &RenderContext, area: Rect) {
    let preview = match ctx.approval_text {
        Some(text) => text,
        None => return,
    };

    let p = ctx.palette;

    // Dim overlay — fill the entire area with a semi-transparent effect
    let overlay = Rect {
        x: area.width / 6,
        y: area.height / 4,
        width: area.width * 2 / 3,
        height: area.height / 3,
    };

    // Dim background block (covers everything behind the popup)
    let dim_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.overlay));
    frame.render_widget(Clear, overlay);
    frame.render_widget(dim_block, overlay);

    // Inner popup block (slightly smaller, centered within overlay)
    let inner = overlay.inner(ratatui::layout::Margin::new(1, 0));
    frame.render_widget(Clear, inner);

    let block = Block::default()
        .title(Span::styled(
            " [!] Tool Approval Required ",
            Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(p.warning))
        .style(Style::default().bg(p.overlay));

    // Preview text (truncated)
    let text = vec![
        Line::from(Span::styled(
            "The following tool requires your approval:",
            Style::default().fg(p.comment),
        )),
        Line::from(""),
        Line::from(Span::styled(
            truncate_preview(preview, 500),
            Style::default()
                .fg(p.foreground)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press (y)es to allow · (n)o to deny · (a)lways allow for this session",
            Style::default().fg(p.warning),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(p.overlay));
    frame.render_widget(block, overlay);
    frame.render_widget(paragraph, inner);
}

/// Truncate preview text to max_chars with a notice.
fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let byte_end = text
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        format!(
            "{}…\n\nPreview truncated ({} chars shown of {})",
            &text[..byte_end],
            max_chars,
            text.chars().count()
        )
    }
}
