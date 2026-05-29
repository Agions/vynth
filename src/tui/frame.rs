//! Frame rendering — layout partitioning and panel drawing

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AgentState, App, InputMode};

/// Draw the entire frame
pub fn draw_frame(frame: &mut ratatui::Frame, app: &App) {
    // Main horizontal split: sidebar | main area
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // Sidebar
            Constraint::Min(0),    // Main area
        ])
        .split(frame.area());

    // Main vertical split: chat | diff | input | status
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Chat area
            Constraint::Length(12), // Diff preview (collapsible)
            Constraint::Length(3),  // Input box
            Constraint::Length(1),  // Status bar
        ])
        .split(h_chunks[1]);

    draw_sidebar(frame, app, h_chunks[0]);
    draw_chat(frame, app, v_chunks[0]);
    draw_diff(frame, app, v_chunks[1]);
    draw_input(frame, app, v_chunks[2]);
    draw_status_bar(frame, app, v_chunks[3]);
}

/// Draw sidebar panel
fn draw_sidebar(frame: &mut ratatui::Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new("  (no files loaded)")
        .block(block)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(paragraph, area);
}

/// Draw chat conversation area
fn draw_chat(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.chat_state.messages {
        let (prefix, color) = match msg.role {
            crate::app::MessageRole::User => ("  You: ", Color::Cyan),
            crate::app::MessageRole::Assistant => ("  AI:  ", Color::Green),
            crate::app::MessageRole::System => ("  Sys: ", Color::Yellow),
            crate::app::MessageRole::Tool => ("  Tool:", Color::Magenta),
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(&msg.content),
        ]));
    }

    // Streaming text
    if app.chat_state.is_streaming {
        lines.push(Line::from(vec![
            Span::styled("  AI:  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(&app.chat_state.streaming_text),
            Span::styled("▌", Style::default().fg(Color::Green)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Ready — type to start chatting (Esc → Normal mode)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Draw diff preview panel
fn draw_diff(frame: &mut ratatui::Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new("  (no pending changes)")
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

/// Draw input box
fn draw_input(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let (title, border_color) = match app.mode {
        InputMode::Insert => (" Input ", Color::Green),
        InputMode::Normal => (" Normal ", Color::Blue),
        InputMode::Command => (" Command ", Color::Yellow),
        InputMode::Search => (" Search ", Color::Magenta),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let hint = match app.mode {
        InputMode::Insert => "Type your message... (Esc → Normal, Ctrl+C → Quit)",
        InputMode::Normal => "i=Insert, :=Cmd, /=Search, q=Quit",
        InputMode::Command => ":",
        InputMode::Search => "/",
    };

    let paragraph = Paragraph::new(hint)
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

/// Draw status bar
fn draw_status_bar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let state_str = match &app.status_bar.agent_state {
        AgentState::Idle => "● Idle",
        AgentState::Thinking => "◌ Thinking...",
        AgentState::RunningTool(name) => &format!("⚙ {}", name),
        AgentState::Error(msg) => &format!("✗ {}", msg),
    };

    let state_color = match &app.status_bar.agent_state {
        AgentState::Idle => Color::Green,
        AgentState::Thinking => Color::Yellow,
        AgentState::RunningTool(_) => Color::Cyan,
        AgentState::Error(_) => Color::Red,
    };

    let _status_text = format!(
        "  {} │ {} │ {}/{} tokens │ sandbox: {}",
        state_str,
        app.status_bar.model_name,
        app.status_bar.tokens_used,
        app.status_bar.tokens_total,
        app.status_bar.sandbox_mode,
    );

    let detail = format!(
        " │ {} │ {}/{} tokens │ sandbox: {}",
        app.status_bar.model_name,
        app.status_bar.tokens_used,
        app.status_bar.tokens_total,
        app.status_bar.sandbox_mode,
    );

    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(state_str, Style::default().fg(state_color)),
        Span::raw(detail),
    ]))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}
