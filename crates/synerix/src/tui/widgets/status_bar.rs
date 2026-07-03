//! Status bar — mode indicator, agent state, model, goal, sandbox, token budget.
//!
//! Segmented layout with elapsed timer, progress bar, and visual separators.
//! Professional design with grouped segments and refined typography.

use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AgentState, InputMode};
use crate::config::SandboxMode;
use crate::tui::activity_label::agent_activity_label;
use crate::tui::animation::animated_dots;
use crate::tui::widgets::primitives::RenderContext;

// ── Separators ─────────────────────────────────────────────────────────────

/// Major segment separator (heavy)
const SEP_MAJOR: &str = " │ ";
/// Minor separator within a group (light)

// ── Input mode colors ──────────────────────────────────────────────────────

fn mode_fg(mode: &InputMode, p: &crate::tui::theme::ThemeReadGuard) -> Color {
    match mode {
        InputMode::Normal => p.accent,
        InputMode::Insert => p.success,
        InputMode::Command => p.warning,
        InputMode::Search => p.chat_tool,
    }
}

// ── Elapsed timer ──────────────────────────────────────────────────────────

/// Format a duration as compact elapsed time with zero-padding.
///
/// Examples: `0s`, `1m 30s`, `2h 03m 09s`
fn format_elapsed(dur: Duration) -> String {
    let total_secs = dur.as_secs();
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {:02}s", mins, secs)
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        format!("{}h {:02}m {:02}s", hours, mins, secs)
    }
}

// ── Segment builders ───────────────────────────────────────────────────────

/// Build the mode indicator span with bracket styling.
fn mode_span(label: &str, p: &crate::tui::theme::ThemeReadGuard) -> Span<'static> {
    Span::styled(
        format!("[{label}]"),
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
    )
}

/// Build the input mode span.
fn input_mode_span(mode: &InputMode, p: &crate::tui::theme::ThemeReadGuard) -> Span<'static> {
    let label = match mode {
        InputMode::Normal => "NORMAL",
        InputMode::Insert => "INSERT",
        InputMode::Command => "CMD",
        InputMode::Search => "SEARCH",
    };
    Span::styled(
        label.to_string(),
        Style::default()
            .fg(mode_fg(mode, p))
            .add_modifier(Modifier::BOLD),
    )
}

/// Build the agent state span with activity label.
fn agent_state_span(
    state: &AgentState,
    frame: u64,
    coding_mode: crate::app::CodingMode,
    p: &crate::tui::theme::ThemeReadGuard,
) -> Span<'static> {
    match state {
        AgentState::Idle => Span::styled(
            "idle".to_string(),
            Style::default().fg(p.success),
        ),
        AgentState::Thinking => {
            let label = agent_activity_label(state, coding_mode);
            Span::styled(
                format!("{}{}", label, animated_dots(frame)),
                Style::default().fg(p.warning),
            )
        }
        AgentState::RunningTool(name) => {
            let display = if name.len() > 10 {
                format!("tool {}..{}", &name[..10], animated_dots(frame))
            } else {
                format!("tool {name}{}", animated_dots(frame))
            };
            Span::styled(
                display,
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
            )
        }
        AgentState::Error(msg) => {
            let display = if msg.len() > 16 {
                format!("error {}..", &msg[..16])
            } else {
                format!("error {msg}")
            };
            Span::styled(display, Style::default().fg(p.error))
        }
    }
}

/// Build a sandbox mode span.
fn sandbox_span(mode: &SandboxMode, p: &crate::tui::theme::ThemeReadGuard) -> Span<'static> {
    let label = match mode {
        SandboxMode::Auto => "auto",
        SandboxMode::Confirm => "confirm",
        SandboxMode::PreviewOnly => "preview",
    };
    Span::styled(
        format!("sandbox:{label}"),
        Style::default().fg(p.muted_fg),
    )
}

// Pre-built progress bar lookup: 13 entries (0-12 filled blocks), each 12 chars wide.
// Avoids allocating a new String every frame.
static PROGRESS_BARS: [&str; 13] = {
    const EMPTY: &str = "░░░░░░░░░░░░";
    const FULL: &str = "████████████";
    // Partial fill patterns for 1-11 filled blocks
    const F1: &str = "█░░░░░░░░░░░";
    const F2: &str = "██░░░░░░░░░░";
    const F3: &str = "███░░░░░░░░░";
    const F4: &str = "████░░░░░░░░";
    const F5: &str = "█████░░░░░░░";
    const F6: &str = "██████░░░░░░";
    const F7: &str = "███████░░░░░";
    const F8: &str = "████████░░░░";
    const F9: &str = "█████████░░░";
    const F10: &str = "██████████░░";
    const F11: &str = "███████████░";
    [EMPTY, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, FULL]
};

/// Build a token budget progress bar with smooth block characters.
///
/// Uses Unicode block elements: `█▓▒░` for a polished look.
/// Colored by usage: <50% green, 50-80% yellow, >80% red.
fn token_progress_bar(used: usize, total: usize, p: &crate::tui::theme::ThemeReadGuard) -> Span<'static> {
    if total == 0 {
        return Span::styled(
            "tokens: --".to_string(),
            Style::default().fg(p.muted_fg),
        );
    }
    let ratio = used as f64 / total as f64;
    let filled = (ratio * 12.0).round().min(12.0) as usize;
    let bar = PROGRESS_BARS[filled];

    let color = if ratio < 0.5 {
        p.success
    } else if ratio < 0.8 {
        p.warning
    } else {
        p.error
    };

    Span::styled(
        format!("tok {bar} {:.0}%", ratio * 100.0),
        Style::default().fg(color),
    )
}

// ── Main render ────────────────────────────────────────────────────────────

/// Render the status bar into the given area.
///
/// Layout: [mode] │ [input] │ [agent + timer] │ [model] │ [goal] │ [sandbox] │ [tokens]
pub fn render_status_bar(frame: &mut Frame, ctx: &RenderContext, area: Rect) {
    let p = ctx.palette;
    let mut spans: Vec<Span> = Vec::with_capacity(24);

    // ── Mode indicator ─────────────────────────────────────────────────────
    let mode_label = ctx.coding_mode.plain_label();
    spans.push(mode_span(mode_label, &p));
    spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));

    // ── Input mode ─────────────────────────────────────────────────────────
    spans.push(input_mode_span(&ctx.input_mode, &p));
    spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));

    // ── Agent state + elapsed timer ────────────────────────────────────────
    let is_busy = !matches!(ctx.agent_state, AgentState::Idle);
    spans.push(agent_state_span(
        ctx.agent_state,
        ctx.anim_frame,
        ctx.coding_mode,
        &p,
    ));

    // Elapsed timer — shows when agent is busy
    if is_busy {
        if let Some(start) = ctx.agent_start {
            let elapsed = Instant::now().duration_since(start);
            spans.push(Span::styled(
                format!("({})", format_elapsed(elapsed)),
                Style::default().fg(p.comment),
            ));
        }
    }
    spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));

    // ── Model name ─────────────────────────────────────────────────────────
    spans.push(Span::styled(
        format!("model:{}", ctx.model_name),
        Style::default().fg(p.muted_fg),
    ));
    spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));

    // ── Goal indicator ─────────────────────────────────────────────────────
    if ctx.goal_active {
        spans.push(Span::styled(
            format!("goal:{}", ctx.goal_duration),
            Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));
    }

    // ── Sandbox mode ───────────────────────────────────────────────────────
    spans.push(sandbox_span(&ctx.sandbox_mode, &p));
    spans.push(Span::styled(SEP_MAJOR, Style::default().fg(p.divider_color)));

    // ── Token progress bar ─────────────────────────────────────────────────
    spans.push(token_progress_bar(
        ctx.tokens_used,
        ctx.tokens_total,
        &p,
    ));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(p.surface).fg(p.foreground));
    frame.render_widget(paragraph, area);
}
