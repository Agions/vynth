//! Chat area widget — renders the conversation with tool calls and scroll offset
//! Uses rounded borders and theme-aware colors for a polished look.

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
        .title_style(
            Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.chat_state.messages {
        let (prefix, color) = match msg.role {
            MessageRole::User => ("  ▸ You:   ", p.chat_user),
            MessageRole::Assistant => ("  ◆ AI:    ", p.chat_assistant),
            MessageRole::System => ("  ◇ Sys:   ", p.chat_system),
            MessageRole::Tool => ("  ⚙ Tool:  ", p.chat_tool),
        };

        let role_span = Span::styled(
            prefix,
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        );

        lines.push(Line::from(vec![
            role_span,
            Span::styled(&msg.content, Style::default().fg(p.foreground)),
        ]));

        // Render tool calls for this message
        for tc in &msg.tool_calls {
            let args_preview = if tc.args_preview.len() > 60 {
                format!("{}…", &tc.args_preview[..60])
            } else {
                tc.args_preview.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("    └─ ", Style::default().fg(p.muted_fg)),
                Span::styled(
                    &tc.name,
                    Style::default()
                        .fg(p.accent)
                        .add_modifier(Modifier::BOLD),
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
                let result_preview = if result.len() > 80 {
                    format!("{}…", &result[..80])
                } else {
                    result.clone()
                };
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        format!("{} {}", icon, result_preview),
                        Style::default().fg(result_color),
                    ),
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

    // Apply scroll offset
    let scroll_offset = app.chat_state.scroll_offset;
    let visible_lines: Vec<Line> = if lines.len() > inner_height {
        let total = lines.len();
        let end = total.saturating_sub(scroll_offset);
        let start = end.saturating_sub(inner_height);
        lines[start..end].to_vec()
    } else {
        lines
    };

    let paragraph = Paragraph::new(visible_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
