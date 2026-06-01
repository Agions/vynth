//! Status bar widget — enhanced with mode, agent state, and metrics display

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AgentState, App, InputMode};

/// Dark status background color — matches the dark theme's status_bg
const STATUS_BG: Color = Color::Rgb(40, 42, 58);

/// Format token count for compact display (e.g., 1234 → "1.2k", 128000 → "128k")
pub fn format_tokens(count: usize) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 10_000 {
        format!("{}k", count / 1000)
    } else if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        format!("{}", count)
    }
}

/// Get the mode label and its display color
pub fn mode_style(mode: &InputMode) -> (&'static str, Color) {
    match mode {
        InputMode::Normal => (" NORMAL ", STATUS_BG), // dark bg for normal
        InputMode::Insert => (" INSERT ", Color::Rgb(30, 100, 50)), // green bg for insert
        InputMode::Command => (" COMMAND ", Color::Rgb(100, 80, 20)), // yellow bg for command
        InputMode::Search => (" SEARCH ", Color::Rgb(80, 40, 100)), // purple bg for search
    }
}

/// Get the mode indicator foreground color
pub fn mode_fg(mode: &InputMode) -> Color {
    match mode {
        InputMode::Normal => Color::Rgb(125, 207, 255), // cyan
        InputMode::Insert => Color::Rgb(158, 206, 121), // green
        InputMode::Command => Color::Rgb(224, 175, 104), // yellow
        InputMode::Search => Color::Rgb(187, 154, 247), // purple
    }
}

/// Render the enhanced status bar into the given area
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (mode_label, mode_bg) = mode_style(&app.mode);
    let mode_fg_color = mode_fg(&app.mode);

    // Agent state section
    let (state_str, state_color) = match &app.status_bar.agent_state {
        AgentState::Idle => (" ● Idle ", Color::Rgb(158, 206, 121)),
        AgentState::Thinking => (" ◌ Thinking… ", Color::Rgb(224, 175, 104)),
        AgentState::RunningTool(name) => {
            // Truncate tool name if too long
            let display_name = if name.len() > 15 {
                format!("{}…", &name[..15])
            } else {
                name.clone()
            };
            // We return a format string, handled below
            let _ = display_name;
            (" tool ", Color::Rgb(125, 207, 255)) // placeholder
        }
        AgentState::Error(msg) => {
            let display_msg = if msg.len() > 20 {
                format!("{}…", &msg[..20])
            } else {
                msg.clone()
            };
            let _ = display_msg;
            (" error ", Color::Rgb(247, 118, 142)) // placeholder
        }
    };

    // Build spans
    let separator = Span::styled(
        " │ ",
        Style::default().fg(Color::Rgb(86, 92, 116)).bg(STATUS_BG),
    );

    let mut spans: Vec<Span> = Vec::new();

    // Left: Mode indicator with background
    spans.push(Span::styled(
        mode_label,
        Style::default()
            .fg(mode_fg_color)
            .bg(mode_bg)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(separator.clone());

    // Center-left: Agent state
    match &app.status_bar.agent_state {
        AgentState::RunningTool(name) => {
            let display_name = if name.len() > 15 {
                format!("{}…", &name[..15])
            } else {
                name.clone()
            };
            spans.push(Span::styled(
                format!(" ⚙ {} ", display_name),
                Style::default()
                    .fg(Color::Rgb(125, 207, 255))
                    .bg(STATUS_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        AgentState::Error(msg) => {
            let display_msg = if msg.len() > 20 {
                format!("{}…", &msg[..20])
            } else {
                msg.clone()
            };
            spans.push(Span::styled(
                format!(" ✗ {} ", display_msg),
                Style::default().fg(Color::Rgb(247, 118, 142)).bg(STATUS_BG),
            ));
        }
        _ => {
            spans.push(Span::styled(
                state_str,
                Style::default().fg(state_color).bg(STATUS_BG),
            ));
        }
    }

    spans.push(separator.clone());

    // Center: Token usage
    let tokens_used_str = format_tokens(app.status_bar.tokens_used);
    let tokens_total_str = format_tokens(app.status_bar.tokens_total);

    // Color tokens based on usage percentage
    let token_color = if app.status_bar.tokens_total > 0 {
        let pct = (app.status_bar.tokens_used as f64 / app.status_bar.tokens_total as f64) * 100.0;
        if pct > 80.0 {
            Color::Rgb(247, 118, 142) // red - high usage
        } else if pct > 60.0 {
            Color::Rgb(224, 175, 104) // yellow - moderate usage
        } else {
            Color::Rgb(160, 170, 210) // muted - normal
        }
    } else {
        Color::Rgb(160, 170, 210)
    };

    spans.push(Span::styled(
        format!(" {} / {} tokens ", tokens_used_str, tokens_total_str),
        Style::default().fg(token_color).bg(STATUS_BG),
    ));

    spans.push(separator.clone());

    // Right: Model name
    spans.push(Span::styled(
        format!(" {} ", app.status_bar.model_name),
        Style::default().fg(Color::Rgb(160, 170, 210)).bg(STATUS_BG),
    ));

    spans.push(separator.clone());

    // Goal indicator — shown before sandbox
    if app.status_bar.goal_active {
        let goal_duration = app.goal_state.duration_str();
        spans.push(Span::styled(
            format!(" ◎ {} ", goal_duration),
            Style::default()
                .fg(Color::Rgb(224, 175, 104))  // yellow
                .bg(STATUS_BG)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(separator.clone());
    }

    // Far right: Sandbox mode with icon
    let sandbox_icon = match app.status_bar.sandbox_mode.to_lowercase().as_str() {
        "auto" => "⚡",
        "confirm" => "🛡",
        "previewonly" | "preview_only" => "👁",
        _ => "🔒",
    };
    spans.push(Span::styled(
        format!(" {} {} ", sandbox_icon, app.status_bar.sandbox_mode),
        Style::default().fg(Color::Rgb(140, 148, 180)).bg(STATUS_BG),
    ));

    let line = Line::from(spans);
    let paragraph =
        Paragraph::new(line).style(Style::default().bg(STATUS_BG).fg(Color::Rgb(192, 202, 245)));

    frame.render_widget(paragraph, area);
}
