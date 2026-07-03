//! Chat area widget — terminal-agent conversation transcript.
//!
//! Message cards with role-colored left border, tool call visualization,
//! streaming status indicator, and scroll offset support.

use std::borrow::Cow;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::app::MessageRole;
use crate::tui::activity_label::agent_activity_label;
use crate::tui::animation::{animated_dots, spinner_frame};
use crate::tui::text::render_plain_text;
use crate::tui::widgets::primitives::RenderContext;


// ── Message card helpers ────────────────────────────────────────────────────

/// Left border character for message cards (OpenCode pattern).
const CARD_BORDER: &str = "│";

// Role icons for message cards
const ICON_USER: &str = ">";
const ICON_ASSISTANT: &str = "●";
const ICON_SYSTEM: &str = "!";
const ICON_TOOL: &str = "⚙";

/// Role colors for message card left border.
fn role_border_color(role: &MessageRole, p: crate::tui::theme::ThemeReadGuard) -> Color {
    match role {
        MessageRole::User => p.chat_user,
        MessageRole::Assistant => p.chat_assistant,
        MessageRole::System => p.warning,
        MessageRole::Tool => p.chat_tool,
    }
}

/// Truncate text to `max_chars` with ellipsis.
fn truncate(text: &str, max_chars: usize) -> Cow<'_, str> {
    if text.chars().count() > max_chars {
        Cow::Owned(format!("{}…", text.chars().take(max_chars).collect::<String>()))
    } else {
        Cow::Borrowed(text)
    }
}

// ── Main render ─────────────────────────────────────────────────────────────

/// Render the chat conversation area with tool call rendering and scroll offset.
pub fn render(area: Rect, frame: &mut ratatui::Frame, ctx: &RenderContext) {
    let p = ctx.palette;
    let inner_height = area.height as usize;

    // Pre-allocate capacity based on estimated line count
    let estimated: usize = ctx
        .chat_messages
        .iter()
        .map(|m| 2 + m.tool_calls.len() * 3)
        .sum();
    let mut lines: Vec<Line> = Vec::with_capacity((estimated + 8).max(2));

    // ── Header with gradient ──────────────────────────────────────────
    let header_bg = crate::tui::theme::Gradient::new(p.accent, p.surface, 1);

    let mut header_spans = vec![Span::styled(
        format!(" {} ", "◈ Synerix "),
        Style::default()
            .fg(p.background)
            .bg(header_bg.at(0, 1))
            .add_modifier(Modifier::BOLD),
    )];

    // Version badge
    header_spans.push(Span::styled(
        format!(" v{} ", env!("CARGO_PKG_VERSION")),
        Style::default().fg(p.muted_fg).bg(p.title_bar_bg),
    ));

    // Mode badge
    header_spans.push(Span::styled(
        format!(" {} ", ctx.coding_mode.label()),
        Style::default().fg(p.accent).bg(p.title_bar_bg),
    ));

    // Model badge
    header_spans.push(Span::styled(
        format!(" {} ", ctx.model_name),
        Style::default().fg(p.muted_fg).bg(p.title_bar_bg),
    ));

    lines.push(Line::from(header_spans));

    // Gradient separator under header (cached by width to avoid per-frame allocation)
    let sep_spans = crate::tui::theme::cached_gradient_separator(area.width);
    lines.push(Line::from(sep_spans));

    // ── Welcome or messages ───────────────────────────────────────────
    if ctx.chat_messages.is_empty()
        && ctx.streaming_text.is_empty()
        && !matches!(ctx.agent_state, crate::app::AgentState::Thinking)
    {
        render_welcome(&mut lines, ctx);
    } else {
        for (msg_idx, msg) in ctx.chat_messages.iter().enumerate() {
            // Message separator (OpenCode pattern: spacing between cards)
            if msg_idx > 0 {
                lines.push(Line::raw(""));
            }

            let role_color = match msg.role {
                MessageRole::User => p.chat_user,
                MessageRole::Assistant => p.chat_assistant,
                MessageRole::System => p.warning,
                MessageRole::Tool => p.chat_tool,
            };

            // Message card with left border (OpenCode pattern)
            let card_border_color = role_border_color(&msg.role, p);
            let prefix = match msg.role {
                MessageRole::User => format!("{} ", ICON_USER),
                MessageRole::Assistant => format!("{} ", ICON_ASSISTANT),
                MessageRole::System => format!("{} ", ICON_SYSTEM),
                MessageRole::Tool => format!("{} ", ICON_TOOL),
            };

            let display_content = render_plain_text(&msg.content);
            let mut content_lines = display_content.lines();
            if let Some(first) = content_lines.next() {
                // First line: border + prefix + content
                if !prefix.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(CARD_BORDER, Style::default().fg(card_border_color)),
                        Span::styled(
                            prefix,
                            Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(first.to_string(), Style::default().fg(p.foreground)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(CARD_BORDER, Style::default().fg(card_border_color)),
                        Span::styled(first.to_string(), Style::default().fg(p.foreground)),
                    ]));
                }
                // Continuation lines: border + indent
                for content_line in content_lines {
                    lines.push(Line::from(vec![
                        Span::styled(CARD_BORDER, Style::default().fg(card_border_color)),
                        Span::raw("  "),
                        Span::styled(content_line.to_string(), Style::default().fg(p.foreground)),
                    ]));
                }
            } else if !prefix.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(CARD_BORDER, Style::default().fg(card_border_color)),
                    Span::styled(prefix, Style::default().fg(role_color).add_modifier(Modifier::BOLD)),
                ]));
            }

            // Render tool calls for this message (Codex pattern)
            for tc in &msg.tool_calls {
                let args_preview = truncate(&tc.args_preview, 50);

                // Tool call header with status indicator
                let (status_icon, status_color) = if tc.result.is_some() {
                    if tc.is_error {
                        ("✗", p.error)
                    } else {
                        ("✓", p.success)
                    }
                } else {
                    // Still running — show spinner
                    let spin = spinner_frame(ctx.anim_frame);
                    (spin, p.accent)
                };

                lines.push(Line::from(vec![
                    Span::styled(CARD_BORDER, Style::default().fg(p.comment)),
                    Span::styled(
                        format!(" {} ", status_icon),
                        Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("tool:{}", tc.name),
                        Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("({})", args_preview),
                        Style::default().fg(p.muted_fg),
                    ),
                ]));

                // Tool result (if available) — Codex pattern: truncated, dimmed
                if let Some(ref result) = tc.result {
                    let result_preview = truncate(result, 80);
                    let result_color = if tc.is_error { p.error } else { p.comment };
                    lines.push(Line::from(vec![
                        Span::styled(CARD_BORDER, Style::default().fg(p.comment)),
                        Span::styled("  ", Style::default().fg(p.comment)),
                        Span::styled(
                            result_preview.to_string(),
                            Style::default().fg(result_color).add_modifier(Modifier::DIM),
                        ),
                    ]));
                }
            }
        }
    }

    // ── Streaming / thinking indicator ────────────────────────────────
    if ctx.is_streaming {
        let dots = animated_dots(ctx.anim_frame);
        let streaming_text = render_plain_text(ctx.streaming_text);
        lines.push(Line::from(vec![
            Span::styled(CARD_BORDER, Style::default().fg(p.chat_assistant)),
            Span::raw(streaming_text),
            Span::styled(dots, Style::default().fg(p.chat_assistant)),
            Span::styled(
                " |",
                Style::default()
                    .fg(p.streaming_cursor)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));
    } else if matches!(ctx.agent_state, crate::app::AgentState::Thinking) {
        let label = agent_activity_label(ctx.agent_state, ctx.coding_mode);
        let spin = spinner_frame(ctx.anim_frame);
        lines.push(Line::from(vec![
            Span::styled(CARD_BORDER, Style::default().fg(p.warning)),
            Span::styled(
                format!("{}{}{}", spin, label, animated_dots(ctx.anim_frame)),
                Style::default().fg(p.warning),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Ready. Ask for a change, run a workflow, or inspect the workspace.",
            Style::default().fg(p.muted_fg),
        )));
    }

    // ── Scroll & render ───────────────────────────────────────────────
    let scroll_offset = if ctx.is_streaming { 0 } else { ctx.scroll_offset };

    // Header + separator occupy 2 lines; remaining height is for messages.
    let header_lines = 2usize;
    let available = inner_height.saturating_sub(header_lines);

    let paragraph = if lines.len() > available {
        let total = lines.len();
        // scroll_offset = 0 shows the newest messages at the bottom.
        // Larger values reveal older history toward the top.
        let end = total.saturating_sub(scroll_offset);
        let start = end.saturating_sub(available);
        let visible: Vec<_> = lines.drain(start..end.min(total)).collect();
        Paragraph::new(visible)
            .block(Block::default().style(Style::default().bg(p.surface)))
            .wrap(Wrap { trim: false })
    } else {
        Paragraph::new(lines)
            .block(Block::default().style(Style::default().bg(p.surface)))
            .wrap(Wrap { trim: false })
    };

    frame.render_widget(paragraph, area);
}

// ── Welcome screen ──────────────────────────────────────────────────────────

/// Render the welcome screen when no messages exist.
///
/// Design: professional dashboard-style layout with logo banner, status cards,
/// and shortcut grid. Uses box-drawing characters for a polished terminal look.
fn render_welcome<'a>(lines: &mut Vec<Line<'a>>, ctx: &RenderContext) {
    let p = ctx.palette;
    let _width = ctx.palette; // kept for adaptive layout parity

    // ── Logo banner with gradient ──────────────────────────────────────
    let logo_text = " Synerix ";
    let version_text = format!(" v{} ", env!("CARGO_PKG_VERSION"));
    let mode_text = format!(" {} ", ctx.coding_mode.label());

    lines.push(Line::from(vec![
        Span::styled(
            logo_text,
            Style::default()
                .fg(p.background)
                .bg(p.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            version_text,
            Style::default()
                .fg(p.muted_fg)
                .bg(p.title_bar_bg),
        ),
        Span::styled(
            mode_text,
            Style::default()
                .fg(p.accent)
                .bg(p.title_bar_bg),
        ),
    ]));

    // Gradient separator
    lines.push(Line::from(vec![
        Span::styled(
            "─".repeat(4),
            Style::default().fg(p.accent),
        ),
        Span::styled(
            "─".repeat(10),
            Style::default().fg(p.divider_color),
        ),
        Span::styled(
            "─".repeat(20),
            Style::default().fg(p.border),
        ),
    ]));

    lines.push(Line::raw(""));

    // ── Status cards row ───────────────────────────────────────────────
    // Card 1: API Status
    let api_label = if ctx.api_key_configured { "● API" } else { "○ API" };
    let api_value = if ctx.api_key_configured {
        "configured"
    } else {
        "not configured"
    };
    let api_color = if ctx.api_key_configured { p.success } else { p.warning };

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<12}", api_label),
            Style::default().fg(api_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<20}", api_value),
            Style::default().fg(p.muted_fg),
        ),
    ]));

    // Card 2: Model
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<12}", "● Model"),
            Style::default().fg(p.chat_assistant).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<20}", ctx.model_name),
            Style::default().fg(p.muted_fg),
        ),
    ]));

    // Card 3: Mode
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<12}", "● Mode"),
            Style::default().fg(p.chat_user).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<20}", ctx.coding_mode.label()),
            Style::default().fg(p.muted_fg),
        ),
    ]));

    lines.push(Line::raw(""));

    // ── Divider ───────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "─".repeat(40),
        Style::default().fg(p.border),
    )));

    lines.push(Line::raw(""));

    // ── Shortcut grid (two columns) ───────────────────────────────────
    let shortcuts_left = [
        ("/help", "show commands"),
        ("/model", "list models"),
        ("Tab", "switch mode"),
    ];

    let shortcuts_right = [
        ("/goal", "set target"),
        ("/clear", "clear chat"),
        ("q", "quit"),
    ];

    for ((left_key, left_desc), (right_key, right_desc)) in
        shortcuts_left.iter().zip(shortcuts_right.iter())
    {
        let left_line = Line::from(vec![
            Span::styled(
                format!("{:<10}", left_key),
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*left_desc, Style::default().fg(p.muted_fg)),
        ]);

        let right_line = Line::from(vec![
            Span::styled(
                format!("{:<10}", right_key),
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*right_desc, Style::default().fg(p.muted_fg)),
        ]);

        // Combine left and right into a single line with spacing
        let mut combined_spans = Vec::new();
        combined_spans.push(Span::raw("  "));
        combined_spans.extend(left_line.spans);
        combined_spans.push(Span::raw("    "));
        combined_spans.extend(right_line.spans);
        lines.push(Line::from(combined_spans));
    }

    lines.push(Line::raw(""));

    // ── Footer hint ───────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "  Press / to see all commands  ·  Start typing to chat",
        Style::default().fg(p.comment).add_modifier(Modifier::ITALIC),
    )));
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncate_short_text() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_text() {
        let result = truncate("hello world foo bar", 10);
        assert!(result.chars().count() <= 11); // 10 chars + ellipsis
        assert!(result.ends_with("…"));
    }
}
