//! Status bar widget — enhanced with mode, agent state, and metrics display

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
        CodingMode::Plan => Color::Rgb(40, 60, 100), // blue-ish
        CodingMode::Act => Color::Rgb(60, 100, 60),  // green-ish
        CodingMode::Chat => Color::Rgb(80, 50, 100), // purple-ish
        CodingMode::Architect => Color::Rgb(100, 80, 60), // brown-ish
        CodingMode::Vibe => Color::Rgb(80, 120, 130), // teal — flow state
    }
}

/// Render the enhanced status bar into the given area
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let p = theme::current_palette();
    let (_mode_label, _mode_bg) = mode_style(&app.mode);
    let _mode_fg_color = mode_fg(&app.mode);

    // Build spans
    let separator = Span::styled(" │ ", Style::default().fg(p.comment).bg(STATUS_BG));

    let mut spans: Vec<Span> = Vec::new();

    // Left: Coding mode badge
    let cm_bg = coding_mode_style(&app.coding_mode);
    spans.push(Span::styled(
        format!(" {} ", app.coding_mode.label()),
        Style::default()
            .fg(Color::Rgb(220, 230, 255))
            .bg(cm_bg)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(separator.clone());

    // Input mode indicator
    let (mode_label, mode_bg_color) = mode_style(&app.mode);
    let mode_fg_color = mode_fg(&app.mode);
    spans.push(Span::styled(
        format!(" {} ", mode_label.trim()),
        Style::default()
            .fg(mode_fg_color)
            .bg(mode_bg_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(separator.clone());

    // Agent state
    let agent_span = match &app.status_bar.agent_state {
        AgentState::Idle => Span::styled(" ● Idle ", Style::default().fg(p.success).bg(STATUS_BG)),
        AgentState::Thinking => Span::styled(
            " ◌ Thinking… ",
            Style::default().fg(p.warning).bg(STATUS_BG),
        ),
        AgentState::RunningTool(name) => {
            let display_name = if name.len() > 15 {
                format!("{}…", &name[..15])
            } else {
                name.clone()
            };
            Span::styled(
                format!(" ⚙ {} ", display_name),
                Style::default()
                    .fg(p.accent)
                    .bg(STATUS_BG)
                    .add_modifier(Modifier::BOLD),
            )
        }
        AgentState::Error(msg) => {
            let display_msg = if msg.len() > 20 {
                format!("{}…", &msg[..20])
            } else {
                msg.clone()
            };
            Span::styled(
                format!(" ✗ {} ", display_msg),
                Style::default().fg(p.error).bg(STATUS_BG),
            )
        }
    };
    spans.push(agent_span);
    spans.push(separator.clone());

    // Token usage with color coding
    let tokens_used_str = format_tokens(app.status_bar.tokens_used);
    let tokens_total_str = format_tokens(app.status_bar.tokens_total);

    let token_color = if app.status_bar.tokens_total > 0 {
        let pct = (app.status_bar.tokens_used as f64 / app.status_bar.tokens_total as f64) * 100.0;
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

    spans.push(Span::styled(
        format!(" {} / {} tokens ", tokens_used_str, tokens_total_str),
        Style::default().fg(token_color).bg(STATUS_BG),
    ));
    spans.push(separator.clone());

    // Model name
    spans.push(Span::styled(
        format!(" {} ", app.status_bar.model_name),
        Style::default().fg(p.muted_fg).bg(STATUS_BG),
    ));
    spans.push(separator.clone());

    // Goal indicator
    if app.status_bar.goal_active {
        let goal_duration = app.goal_state.duration_str();
        spans.push(Span::styled(
            format!(" ◎ {} ", goal_duration),
            Style::default()
                .fg(p.warning)
                .bg(STATUS_BG)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(separator.clone());
    }

    // Sandbox mode with icon
    let sandbox_icon = match app.status_bar.sandbox_mode.to_lowercase().as_str() {
        "auto" => "⚡",
        "confirm" => "🛡",
        "previewonly" | "preview_only" => "👁",
        _ => "🔒",
    };
    spans.push(Span::styled(
        format!(" {} {} ", sandbox_icon, app.status_bar.sandbox_mode),
        Style::default().fg(p.muted_fg).bg(STATUS_BG),
    ));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(STATUS_BG).fg(p.foreground));

    frame.render_widget(paragraph, area);
}
