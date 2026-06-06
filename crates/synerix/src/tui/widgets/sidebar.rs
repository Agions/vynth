//! Sidebar widget — file tree / session list / skills
//! Enhanced with rounded borders, icon indicators, and theme-aware styling.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, SidebarTab};
use crate::tui::theme;

/// Render sidebar panel with tab bar and content area
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
    let p = theme::current_palette();

    // Split area into tab bar (1 row) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // ── Tab bar ──────────────────────────────────────────────────────────
    let tabs = [
        ("📁 Files", SidebarTab::Files),
        ("📋 Sessions", SidebarTab::Sessions),
        ("🧩 Skills", SidebarTab::Skills),
    ];

    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, (label, tab))| {
            let is_active = app.sidebar_state.active_tab == *tab;
            let style = if is_active {
                Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.muted_fg)
            };
            let sep = if i > 0 { " " } else { "" };
            vec![Span::raw(sep), Span::styled(*label, style)]
        })
        .collect();

    let tab_line = Line::from(tab_spans);
    let tab_paragraph = Paragraph::new(tab_line).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_type(theme::BORDER_TYPE)
            .border_style(Style::default().fg(p.border)),
    );
    frame.render_widget(tab_paragraph, chunks[0]);

    // ── Content area ─────────────────────────────────────────────────────
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(Style::default().fg(p.border));

    let content_text = match app.sidebar_state.active_tab {
        SidebarTab::Files => {
            if app.sidebar_state.file_tree.is_empty() {
                format!(
                    "  {}  (no files loaded)",
                    Span::styled("📂", Style::default().fg(p.muted_fg))
                )
            } else {
                let scroll = app.sidebar_state.scroll_offset;
                app.sidebar_state
                    .file_tree
                    .iter()
                    .skip(scroll)
                    .map(|f| {
                        let indent = "  ".repeat(f.depth);
                        let icon = if f.is_dir { "📁" } else { "📄" };
                        format!("{}{} {}", indent, icon, f.name)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        SidebarTab::Sessions => {
            format!(
                "  {}  (no sessions)",
                Span::styled("📄", Style::default().fg(p.muted_fg))
            )
        }
        SidebarTab::Skills => {
            format!(
                "  {}  (no skills loaded)",
                Span::styled("🧩", Style::default().fg(p.muted_fg))
            )
        }
    };

    let paragraph = Paragraph::new(content_text)
        .block(content_block)
        .style(Style::default().fg(p.muted_fg));

    frame.render_widget(paragraph, chunks[1]);
}
