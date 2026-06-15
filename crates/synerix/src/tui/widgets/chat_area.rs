//! Chat area widget — renders the conversation with tool calls and scroll offset
//! Uses rounded borders and theme-aware colors for a polished look.

use std::borrow::Cow;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, FocusedPanel, MessageRole};
use crate::tui::theme;

/// Render the chat conversation area with tool call rendering and scroll offset
pub fn render(area: Rect, frame: &mut ratatui::Frame, app: &App) {
    let p = theme::current_palette();
    let is_focused = app.focused_panel == FocusedPanel::Chat;
    let border_style = if is_focused {
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(p.border)
    };

    let block = Block::default()
        .title(" 💬 Chat ")
        .title_style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize;

    // Pre-allocate capacity based on estimated line count (role line + up to 2 per tool call)
    let estimated: usize = app
        .chat_state
        .messages
        .iter()
        .map(|m| 1 + m.tool_calls.len() * 2)
        .sum();
    let mut lines: Vec<Line> = Vec::with_capacity(estimated.max(2));

    for msg in &app.chat_state.messages {
        let (prefix, color) = match msg.role {
            MessageRole::User => ("  ▸ You:   ", p.chat_user),
            MessageRole::Assistant => ("  ◆ AI:    ", p.chat_assistant),
            MessageRole::System => ("  ◇ Sys:   ", p.chat_system),
            MessageRole::Tool => ("  ⚙ Tool:  ", p.chat_tool),
        };

        let role_span = Span::styled(
            prefix,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );

        lines.push(Line::from(vec![
            role_span,
            Span::styled(&msg.content, Style::default().fg(p.foreground)),
        ]));

        // Render tool calls for this message
        for tc in &msg.tool_calls {
            // Use Cow to avoid unconditional clone for untruncated args
            let args_preview: Cow<'_, str> = if tc.args_preview.len() > 60 {
                Cow::Owned(format!("{}…", &tc.args_preview[..60]))
            } else {
                Cow::Borrowed(&tc.args_preview)
            };
            lines.push(Line::from(vec![
                Span::styled("    └─ ", Style::default().fg(p.muted_fg)),
                Span::styled(
                    &tc.name,
                    Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({})", args_preview),
                    Style::default().fg(p.muted_fg),
                ),
            ]));

            if let Some(ref result) = tc.result {
                let (icon, result_color) = if tc.is_error {
                    ("✗", p.error)
                } else {
                    ("✓", p.success)
                };
                // Use Cow to avoid unconditional clone for untruncated results
                let result_preview: Cow<'_, str> = if result.len() > 80 {
                    Cow::Owned(format!("{}…", &result[..80]))
                } else {
                    Cow::Borrowed(result.as_str())
                };
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(icon, Style::default().fg(result_color)),
                    Span::styled(" ", Style::default().fg(result_color)),
                    Span::styled(result_preview, Style::default().fg(result_color)),
                ]));
            }
        }
    }

    // Streaming text
    if app.chat_state.is_streaming {
        lines.push(Line::from(vec![
            Span::styled(
                "  ◆ AI:    ",
                Style::default()
                    .fg(p.chat_assistant)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.chat_state.streaming_text),
            Span::styled(
                " ▌",
                Style::default()
                    .fg(p.streaming_cursor)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Ready — type to start (Esc = Normal mode)",
            Style::default().fg(p.muted_fg),
        )));
    }

    // Apply scroll offset — avoid clone via split_off + truncate (moves, not copies)
    let scroll_offset = app.chat_state.scroll_offset;
    let paragraph = if lines.len() > inner_height {
        let total = lines.len();
        let end = total.saturating_sub(scroll_offset);
        let start = end.saturating_sub(inner_height);
        let mut visible = lines.split_off(start);
        visible.truncate(end - start);
        Paragraph::new(visible)
            .block(block)
            .wrap(Wrap { trim: false })
    } else {
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
    };

    frame.render_widget(paragraph, area);
}
