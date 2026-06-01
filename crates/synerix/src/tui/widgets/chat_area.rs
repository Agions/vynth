//! Chat area widget — renders the conversation with tool calls and scroll offset

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, FocusedPanel, MessageRole};
use crate::tui::theme;

/// Render the chat conversation area with tool call rendering and scroll offset
pub fn render(area: Rect, frame: &mut ratatui::Frame, app: &App) {
    let block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(
            if app.focused_panel == FocusedPanel::Chat {
                Style::default().fg(theme::COLOR_CYAN)
            } else {
                theme::muted_style()
            },
        );

    let inner_height = area.height.saturating_sub(2) as usize; // subtract top/bottom border
    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.chat_state.messages {
        let (prefix, color) = match msg.role {
            MessageRole::User => ("  You: ", theme::COLOR_CYAN),
            MessageRole::Assistant => ("  AI:  ", theme::COLOR_GREEN),
            MessageRole::System => ("  Sys: ", theme::COLOR_YELLOW),
            MessageRole::Tool => ("  Tool:", theme::COLOR_MAGENTA),
        };

        lines.push(Line::from(vec![
            Span::styled(
                prefix,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(&msg.content),
        ]));

        // Render tool calls for this message
        for tc in &msg.tool_calls {
            // Tool call header: 🔧 tool_name with args preview
            let args_preview = if tc.args_preview.len() > 60 {
                format!("{}…", &tc.args_preview[..60])
            } else {
                tc.args_preview.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("    🔧 ", Style::default().fg(theme::COLOR_CYAN)),
                Span::styled(
                    &tc.name,
                    Style::default()
                        .fg(theme::COLOR_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({})", args_preview),
                    theme::muted_style(),
                ),
            ]));

            // Tool result line
            if let Some(ref result) = tc.result {
                let (icon, result_color) = if tc.is_error {
                    ("❌", theme::COLOR_RED)
                } else {
                    ("✅", theme::COLOR_GREEN)
                };
                let result_preview = if result.len() > 80 {
                    format!("{}…", &result[..80])
                } else {
                    result.clone()
                };
                lines.push(Line::from(vec![
                    Span::raw("       "),
                    Span::styled(icon, Style::default().fg(result_color)),
                    Span::styled(
                        format!(" {}", result_preview),
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
                "  AI:  ",
                Style::default()
                    .fg(theme::COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.chat_state.streaming_text),
            Span::styled("▌", Style::default().fg(theme::COLOR_GREEN)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Ready — type to start chatting (Esc → Normal mode)",
            theme::muted_style(),
        )));
    }

    // Apply scroll offset: skip lines from the top based on scroll_offset
    let scroll_offset = app.chat_state.scroll_offset;
    let visible_lines: Vec<Line> = if lines.len() > inner_height {
        let total = lines.len();
        // scroll_offset = number of lines hidden from bottom (0 = latest at bottom)
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
