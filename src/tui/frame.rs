//! Frame rendering — layout partitioning and panel drawing

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AgentState, App, InputMode, SidebarTab};

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

/// Draw sidebar panel with tab bar
fn draw_sidebar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    // Split area into tab bar (1 row) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Tab bar
    let tabs = [
        ("Files", SidebarTab::Files),
        ("Sessions", SidebarTab::Sessions),
        ("Skills", SidebarTab::Skills),
    ];

    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .map(|(i, (label, tab))| {
            let is_active = app.sidebar_state.active_tab == *tab;
            let style = if is_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let sep = if i > 0 { " " } else { "" };
            vec![
                Span::raw(sep),
                Span::styled(*label, style),
            ]
        })
        .flatten()
        .collect();

    let tab_line = Line::from(tab_spans);
    let tab_paragraph = Paragraph::new(tab_line).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(tab_paragraph, chunks[0]);

    // Content area
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let content_text = match app.sidebar_state.active_tab {
        SidebarTab::Files => {
            if app.sidebar_state.file_tree.is_empty() {
                "  (no files loaded)".to_string()
            } else {
                app.sidebar_state
                    .file_tree
                    .iter()
                    .map(|f| {
                        let indent = "  ".repeat(f.depth);
                        let icon = if f.is_dir { "📁" } else { "📄" };
                        format!("{}{} {}", indent, icon, f.name)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        SidebarTab::Sessions => "  (no sessions)".to_string(),
        SidebarTab::Skills => "  (no skills loaded)".to_string(),
    };

    let paragraph = Paragraph::new(content_text)
        .block(content_block)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(paragraph, chunks[1]);
}

/// Draw chat conversation area with tool call rendering and scroll offset
fn draw_chat(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_height = area.height.saturating_sub(2) as usize; // subtract top/bottom border
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

        // Render tool calls for this message
        for tc in &msg.tool_calls {
            // Tool call header: 🔧 tool_name with args preview
            let args_preview = if tc.args_preview.len() > 60 {
                format!("{}…", &tc.args_preview[..60])
            } else {
                tc.args_preview.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("    🔧 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    &tc.name,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({})", args_preview),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Tool result line
            if let Some(ref result) = tc.result {
                let (icon, result_color) = if tc.is_error {
                    ("❌", Color::Red)
                } else {
                    ("✅", Color::Green)
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

/// Draw diff preview panel
fn draw_diff(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if app.diff_state.content.is_empty() {
        let paragraph = Paragraph::new("  (no pending changes)")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    } else {
        // Compute available inner width for side-by-side column sizing
        let inner_width = area.width.saturating_sub(3) as usize; // border + gutter
        let col_width = inner_width / 2;

        let diff_text = crate::tui::diff_renderer::render_diff(
            &app.diff_state.content,
            crate::tui::diff_renderer::DiffViewMode::SideBySide,
            col_width,
        );

        let paragraph = Paragraph::new(diff_text).block(block);
        frame.render_widget(paragraph, area);
    }
}

/// Draw input box showing actual input buffer with cursor
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

    match app.mode {
        InputMode::Insert => {
            if app.input_buffer.is_empty() {
                let hint = "Type your message... (Esc → Normal, Ctrl+C → Quit)";
                let paragraph = Paragraph::new(hint)
                    .block(block)
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(paragraph, area);
            } else {
                // Show actual input text with cursor
                let cursor_pos = app.input_cursor.min(app.input_buffer.len());
                let before = &app.input_buffer[..cursor_pos];
                let after = &app.input_buffer[cursor_pos..];

                let line = Line::from(vec![
                    Span::styled(before, Style::default().fg(Color::White)),
                    Span::styled(
                        if after.is_empty() { " " } else { &after[..1] },
                        Style::default().bg(Color::White).fg(Color::Black),
                    ),
                    Span::raw(if after.is_empty() { "" } else { &after[1..] }),
                ]);

                let paragraph = Paragraph::new(line)
                    .block(block)
                    .wrap(Wrap { trim: false });
                frame.render_widget(paragraph, area);
            }
        }
        InputMode::Normal => {
            let hint = "i=Insert, :=Cmd, /=Search, q=Quit";
            let paragraph = Paragraph::new(hint)
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(paragraph, area);
        }
        InputMode::Command => {
            let line = Line::from(vec![
                Span::styled(":", Style::default().fg(Color::Yellow)),
                Span::raw(&app.input_buffer),
                Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
        InputMode::Search => {
            let line = Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Magenta)),
                Span::raw(&app.input_buffer),
                Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)),
            ]);
            let paragraph = Paragraph::new(line).block(block);
            frame.render_widget(paragraph, area);
        }
    }
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
