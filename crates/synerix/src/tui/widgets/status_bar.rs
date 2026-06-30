//! Compact terminal status bar.

use std::fmt::Write;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AgentState, App, InputMode};
use crate::config::SandboxMode;
use crate::tui::activity_label::agent_activity_label;
use crate::tui::animation::animated_dots;
use crate::tui::theme;

const STATUS_BG: Color = Color::Rgb(40, 42, 58);
const SEPARATOR: &str = "  ";
const IDLE_LABEL: &str = "idle";

/// Get the mode label and its display color
pub fn mode_style(mode: &InputMode) -> (&'static str, Color) {
    match mode {
        InputMode::Normal => ("normal", STATUS_BG),
        InputMode::Insert => ("insert", STATUS_BG),
        InputMode::Command => ("command", STATUS_BG),
        InputMode::Search => ("search", STATUS_BG),
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

/// Build the agent state span — avoid format! for common cases.
fn agent_state_span(
    state: &AgentState,
    frame: u64,
    coding_mode: crate::coding_modes::CodingMode,
    p: &theme::ColorPalette,
) -> Span<'static> {
    match state {
        AgentState::Idle => Span::styled(IDLE_LABEL, Style::default().fg(p.success).bg(STATUS_BG)),
        AgentState::Thinking => Span::styled(
            format!(
                "{}{}",
                agent_activity_label(state, coding_mode),
                animated_dots(frame)
            ),
            Style::default().fg(p.warning).bg(STATUS_BG),
        ),
        AgentState::RunningTool(name) => {
            let display = if name.len() > 15 {
                let mut buf = String::with_capacity(19);
                let _ = write!(buf, "tool {}…{}", &name[..15], animated_dots(frame));
                buf
            } else {
                let mut buf = String::with_capacity(name.len() + 8);
                let _ = write!(buf, "tool {name}{}", animated_dots(frame));
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
                let _ = write!(buf, "error {}…", &msg[..20]);
                buf
            } else {
                let mut buf = String::with_capacity(msg.len() + 7);
                let _ = write!(buf, "error {msg}");
                buf
            };
            Span::styled(display, Style::default().fg(p.error).bg(STATUS_BG))
        }
    }
}

/// Build a sandbox mode span with a stable lowercase label.
fn sandbox_span(mode: &SandboxMode, p: &theme::ColorPalette) -> Span<'static> {
    let label = match mode {
        SandboxMode::Auto => "auto",
        SandboxMode::Confirm => "confirm",
        SandboxMode::PreviewOnly => "preview_only",
    };
    let mut buf = String::with_capacity(label.len() + 8);
    let _ = write!(buf, "sandbox {label}");
    Span::styled(buf, Style::default().fg(p.muted_fg).bg(STATUS_BG))
}

/// Render the enhanced status bar into the given area
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let p = theme::current_palette();
    let mut spans: Vec<Span> = Vec::with_capacity(16);

    let label = app.coding_mode.plain_label().to_lowercase();
    let mut buf = String::with_capacity(label.len() + 5);
    let _ = write!(buf, "mode {label}");
    spans.push(Span::styled(
        buf,
        Style::default()
            .fg(p.accent)
            .bg(STATUS_BG)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Input mode indicator
    let (mode_label, mode_bg_color) = mode_style(&app.mode);
    let mode_fg_color = mode_fg(&app.mode);
    let mut buf = String::with_capacity(mode_label.len() + 6);
    let _ = write!(buf, "input {mode_label}");
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
    spans.push(agent_state_span(
        &app.status_bar.agent_state,
        app.status_bar.animation_frame,
        app.coding_mode,
        p,
    ));
    spans.push(Span::styled(
        SEPARATOR,
        Style::default().fg(p.comment).bg(STATUS_BG),
    ));

    // Model name
    let mut buf = String::with_capacity(app.status_bar.model_name.len() + 7);
    let _ = write!(buf, "model {}", app.status_bar.model_name);
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
        let mut buf = String::with_capacity(duration.len() + 6);
        let _ = write!(buf, "goal {duration}");
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
    spans.push(sandbox_span(&app.status_bar.sandbox_mode, p));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(STATUS_BG).fg(p.foreground));
    frame.render_widget(paragraph, area);
}
