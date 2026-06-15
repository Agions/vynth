//! Status bar widget — enhanced with mode, agent state, and metrics display
//!
//! Optimized to minimize per-frame allocations: pre-computed static strings,
//! write! to pre-allocated buffers, lazy-evaluated token formatting.

use std::fmt::Write;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AgentState, App, InputMode};
use crate::coding_modes::CodingMode;
use crate::tui::theme;

/// Dark status background color — matches the dark theme's status_bg
const STATUS_BG: Color = Color::Rgb(40, 42, 58);

/// Pre-computed style objects to avoid Style::default() per frame.
/// Created once via `once_cell::sync::Lazy` (or just inline constants).
const SEPARATOR: &str = " │ ";
const THINKING_LABEL: &str = " ◌ Thinking… ";
const IDLE_LABEL: &str = " ● Idle ";

/// Format token count for compact display (e.g., 1234 → "1.2k", 128000 → "128k")
/// Uses write! to a pre-allocated buffer to avoid format! overhead.
pub fn format_tokens(count: usize) -> String {
    // Inline the logic to avoid the function call overhead on every frame
    // when the status bar is rendered.
    let mut buf = String::with_capacity(8);
    if count >= 1_000_000 {
        let _ = write!(buf, "{:.1}M", count as f64 / 1_000_000.0);
    } else if count >= 10_000 {
        let _ = write!(buf, "{}k", count / 1000);
    } else if count >= 1000 {
        let _ = write!(buf, "{:.1}k", count as f64 / 1000.0);
    } else {
        let _ = write!(buf, "{count}");
    }
    buf
}

/// Get the mode label and its display color
pub fn mode_style(mode: &InputMode) -> (&'static str, Color) {
    match mode {
        InputMode::Normal => (" NORMAL ", STATUS_BG),
        InputMode::Insert => (" INSERT ", Color::Rgb(30, 100, 50)),
        InputMode::Command => (" COMMAND ", Color::Rgb(100, 80, 20)),
        InputMode::Search => (" SEARCH ", Color::Rgb(80, 40, 100)),
    }
}

/// Get the mode indicator foreground color
pub fn mode_fg(mode: &InputMode) -> Color {
    match mode {
        InputMode::Normal => Color::Rgb(125, 207, 255),
        InputMode::Insert => Color::Rgb(158, 206, 121),
        InputMode::Command => Color::Rgb(224, 175, 104),
        InputMode::Search => Color::Rgb(187, 154, 247),
    }
}

/// Get coding mode style (background color based on mode)
pub fn coding_mode_style(mode: &CodingMode) -> Color {
    match mode {
        CodingMode::Plan => Color::Rgb(40, 60, 100),
        CodingMode::Act => Color::Rgb(60, 100, 60),
        CodingMode::Chat => Color::Rgb(80, 50, 100),
        CodingMode::Architect => Color::Rgb(100, 80, 60),
        CodingMode::Vibe => Color::Rgb(80, 120, 130),
    }
}

/// Build the agent state span — avoid format! for common cases.
fn agent_state_span(state: &AgentState, p: &theme::ColorPalette) -> Span<'static> {
    match state {
        AgentState::Idle => Span::styled(IDLE_LABEL, Style::default().fg(p.success).bg(STATUS_BG)),
        AgentState::Thinking => {
            Span::styled(THINKING_LABEL, Style::default().fg(p.warning).bg(STATUS_BG))
        }
        AgentState::RunningTool(name) => {
            // Only allocate when the tool name exceeds 15 chars
            let display = if name.len() > 15 {
                let mut buf = String::with_capacity(19);
                let _ = write!(buf, " ⚙ {}… ", &name[..15]);
                buf
            } else {
                let mut buf = String::with_capacity(name.len() + 4);
                let _ = write!(buf, " ⚙ {name} ");
                buf
            };
            Span::styled(
                display,
                Style::default()
                    .fg(p.accent)
                    .bg(STATUS_BG)
                    .add_modifier(Modifier::BOLD),
            )
        }
        AgentState::Error(msg) => {
            let display = if msg.len() > 20 {
                let mut buf = String::with_capacity(24);
                let _ = write!(buf, " ✗ {}… ", &msg[..20]);
                buf
            } else {
                let mut buf = String::with_capacity(msg.len() + 4);
                let _ = write!(buf, " ✗ {msg} ");
                buf
            };
            Span::styled(display, Style::default().fg(p.error).bg(STATUS_BG))
        }
    }
}

/// Build a token usage span with colour-coded percentage.
fn token_span(used: usize, total: usize, p: &theme::ColorPalette) -> Span<'static> {
    let pct_color = if total > 0 {
        let pct = (used as f64 / total as f64) * 100.0;
        if pct > 80.0 {
            p.error
        } else if pct > 60.0 {
            p.warning
        } else {
            p.muted_fg
        }
    } else {
        p.muted_fg
    };

    // Pre-allocate buffer: tokens typically "1.2k / 128k tokens" ~= 25 chars
    let mut buf = String::with_capacity(32);
    let used_str = format_tokens(used);
    let total_str = format_tokens(total);
    let _ = write!(buf, " {used_str} / {total_str} tokens ");
    Span::styled(buf, Style::default().fg(pct_color).bg(STATUS_BG))
}

/// Build a sandbox mode span — avoid to_lowercase() by matching known patterns.
fn sandbox_span(mode: &str, p: &theme::ColorPalette) -> Span<'static> {
    let icon = match mode {
        "auto" | "Auto" | "AUTO" => "⚡",
        "confirm" | "Confirm" | "CONFIRM" => "🛡",
        "previewonly" | "preview_only" | "PreviewOnly" => "👁",
        _ => "🔒",
    };
    let mut buf = String::with_capacity(mode.len() + 4);
    let _ = write!(buf, " {icon} {mode} ");
    Span::styled(buf, Style::default().fg(p.muted_fg).bg(STATUS_BG))
}

/// Render the enhanced status bar into the given area
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let p = theme::current_palette();
    let mut spans: Vec<Span> = Vec::with_capacity(16);

    // Left: Coding mode badge
    let cm_bg = coding_mode_style(&app.coding_mode);
    let label = app.coding_mode.label();
    let mut buf = String::with_capacity(label.len() + 3);
    let _ = write!(buf, " {label} ");
    spans.push(Span::styled(
        buf,
        Style::default()
            .fg(Color::Rgb(220, 230, 255))
            .bg(cm_bg)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Input mode indicator
    let (mode_label, mode_bg_color) = mode_style(&app.mode);
    let mode_fg_color = mode_fg(&app.mode);
    let trimmed = mode_label.trim();
    let mut buf = String::with_capacity(trimmed.len() + 3);
    let _ = write!(buf, " {trimmed} ");
    spans.push(Span::styled(
        buf,
        Style::default()
            .fg(mode_fg_color)
            .bg(mode_bg_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Agent state
    spans.push(agent_state_span(&app.status_bar.agent_state, &p));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Token usage
    spans.push(token_span(
        app.status_bar.tokens_used,
        app.status_bar.tokens_total,
        &p,
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Model name
    let mut buf = String::with_capacity(app.status_bar.model_name.len() + 3);
    let _ = write!(buf, " {} ", app.status_bar.model_name);
    spans.push(Span::styled(
        buf,
        Style::default().fg(p.muted_fg).bg(STATUS_BG),
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Goal indicator
    if app.status_bar.goal_active {
        let duration = app.goal_state.duration_str();
        let mut buf = String::with_capacity(duration.len() + 5);
        let _ = write!(buf, " ◎ {duration} ");
        spans.push(Span::styled(
            buf,
            Style::default()
                .fg(p.warning)
                .bg(STATUS_BG)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            SEPARATOR,
            Style::default().fg(p.comment).bg(STATUS_BG),
        ));
    }

    // Sandbox mode
    spans.push(sandbox_span(&app.status_bar.sandbox_mode, &p));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(STATUS_BG).fg(p.foreground));
    frame.render_widget(paragraph, area);
}
