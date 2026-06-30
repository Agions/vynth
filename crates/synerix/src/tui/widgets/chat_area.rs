//! Chat area widget — terminal-agent conversation transcript.

use std::borrow::Cow;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::app::{App, MessageRole};
use crate::tui::activity_label::agent_activity_label;
use crate::tui::animation::animated_dots;
use crate::tui::text::render_plain_text;
use crate::tui::theme;

const LOGO: &str = "◆ Synerix";
const WELCOME_LOGO: &[&str] = &[
    " ______   ___   _ _____ ____  _____  __",
    "/ ___\\ \\ / / \\ | | ____|  _ \\|_ _\\ \\/ /",
    "\\___ \\\\ V /|  \\| |  _| | |_) || | \\  / ",
    " ___) || | | |\\  | |___|  _ < | | /  \\ ",
    "|____/ |_| |_| \\_|_____|_| \\_\\___/_/\\_\\",
    "                                       ",
];

/// Render the chat conversation area with tool call rendering and scroll offset
pub fn render(area: Rect, frame: &mut ratatui::Frame, app: &App) {
    let p = theme::current_palette();
    let block = Block::default();
    let inner_height = area.height as usize;

    // Pre-allocate capacity based on estimated line count (role line + up to 2 per tool call)
    let estimated: usize = app
        .chat_state
        .messages
        .iter()
        .map(|m| 1 + m.tool_calls.len() * 2)
        .sum();
    let mut lines: Vec<Line> = Vec::with_capacity(estimated.max(2));

    let header = Line::from(vec![
        Span::styled(
            LOGO,
            Style::default()
                .fg(p.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(p.muted_fg),
        ),
        Span::styled("  ", Style::default().fg(p.muted_fg)),
        Span::styled(app.coding_mode.plain_label(), Style::default().fg(p.accent)),
        Span::styled("  ", Style::default().fg(p.muted_fg)),
        Span::styled(&app.status_bar.model_name, Style::default().fg(p.muted_fg)),
    ]);
    lines.push(header);
    lines.push(Line::raw(""));

    if app.chat_state.messages.is_empty()
        && app.chat_state.streaming_text.is_empty()
        && !matches!(app.status_bar.agent_state, crate::app::AgentState::Thinking)
    {
        render_welcome(&mut lines, app, p);
    }

    for msg in &app.chat_state.messages {
        let prefix = message_prefix(&msg.role);
        let color = match msg.role {
            MessageRole::User => p.chat_user,
            MessageRole::Assistant => p.chat_assistant,
            MessageRole::System => p.chat_system,
            MessageRole::Tool => p.chat_tool,
        };

        let display_content = render_plain_text(&msg.content);
        let mut content_lines = display_content.lines();
        if let Some(first) = content_lines.next() {
            if let Some(prefix) = prefix {
                lines.push(Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(first.to_string(), Style::default().fg(p.foreground)),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    first.to_string(),
                    Style::default().fg(p.foreground),
                )));
            }
            for content_line in content_lines {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(content_line.to_string(), Style::default().fg(p.foreground)),
                ]));
            }
        } else if let Some(prefix) = prefix {
            lines.push(Line::from(Span::styled(
                prefix,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
        }

        // Render tool calls for this message
        for tc in &msg.tool_calls {
            // Use Cow to avoid unconditional clone for untruncated args
            let args_preview: Cow<'_, str> = if tc.args_preview.len() > 60 {
                Cow::Owned(format!("{}…", &tc.args_preview[..60]))
            } else {
                Cow::Borrowed(&tc.args_preview)
            };
            lines.push(Line::from(vec![
                Span::styled("  | tool ", Style::default().fg(p.muted_fg)),
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
                    Span::styled("  | ", Style::default().fg(p.muted_fg)),
                    Span::styled(icon, Style::default().fg(result_color)),
                    Span::styled(" ", Style::default().fg(result_color)),
                    Span::styled(result_preview, Style::default().fg(result_color)),
                ]));
            }
        }
        lines.push(Line::raw(""));
    }

    // Streaming text
    if app.chat_state.is_streaming {
        let dots = animated_dots(app.status_bar.animation_frame);
        let streaming_text = render_plain_text(&app.chat_state.streaming_text);
        lines.push(Line::from(vec![
            Span::raw(streaming_text),
            Span::styled(dots, Style::default().fg(p.chat_assistant)),
            Span::styled(
                " ▌",
                Style::default()
                    .fg(p.streaming_cursor)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));
    } else if matches!(app.status_bar.agent_state, crate::app::AgentState::Thinking) {
        let label = agent_activity_label(&app.status_bar.agent_state, app.coding_mode);
        lines.push(Line::from(vec![Span::styled(
            format!("{}{}", label, animated_dots(app.status_bar.animation_frame)),
            Style::default().fg(p.accent),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Ready. Ask for a change, run a workflow, or inspect the workspace.",
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

fn render_welcome<'a>(lines: &mut Vec<Line<'a>>, app: &'a App, p: &theme::ColorPalette) {
    let api_status = if app.settings.llm.api_key.trim().is_empty() {
        "API key: not configured"
    } else {
        "API key: configured"
    };
    for logo_line in WELCOME_LOGO {
        lines.push(Line::from(Span::styled(
            *logo_line,
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("mode  ", Style::default().fg(p.muted_fg)),
        Span::styled(app.coding_mode.plain_label(), Style::default().fg(p.accent)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("model ", Style::default().fg(p.muted_fg)),
        Span::styled(
            app.status_bar.model_name.clone(),
            Style::default().fg(p.foreground),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        api_status,
        Style::default().fg(if app.settings.llm.api_key.trim().is_empty() {
            p.warning
        } else {
            p.success
        }),
    )));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Type a task and press Enter.",
        Style::default().fg(p.foreground),
    )));
    lines.push(Line::from(Span::styled(
        "/help        show commands",
        Style::default().fg(p.muted_fg),
    )));
    lines.push(Line::from(Span::styled(
        "/model list  model presets",
        Style::default().fg(p.muted_fg),
    )));
    lines.push(Line::from(Span::styled(
        "Tab          switch development mode",
        Style::default().fg(p.muted_fg),
    )));
}

fn message_prefix(role: &MessageRole) -> Option<&'static str> {
    match role {
        MessageRole::User => Some("> "),
        MessageRole::Assistant | MessageRole::System => None,
        MessageRole::Tool => Some("tool "),
    }
}

#[cfg(test)]
mod tests {
    use super::message_prefix;
    use crate::app::MessageRole;

    #[test]
    fn message_prefix_hides_internal_role_labels() {
        assert_eq!(message_prefix(&MessageRole::Assistant), None);
        assert_eq!(message_prefix(&MessageRole::System), None);
    }
}
