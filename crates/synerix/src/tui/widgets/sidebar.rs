//! Sidebar widget — file tree / session list / skills
// TODO: Sidebar widget — not yet wired
#![allow(dead_code)]

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, FileEntry, SidebarTab};
use crate::tui::theme;

pub struct Sidebar {
    pub active_tab: SidebarTab,
    pub file_tree: Vec<FileEntry>,
    pub scroll_offset: usize,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active_tab: SidebarTab::Files,
            file_tree: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn switch_tab(&mut self, tab: SidebarTab) {
        self.active_tab = tab;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}

/// Render sidebar panel with tab bar
pub fn render(area: Rect, frame: &mut Frame, app: &App) {
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
        .flat_map(|(i, (label, tab))| {
            let is_active = app.sidebar_state.active_tab == *tab;
            let style = if is_active {
                Style::default()
                    .fg(theme::COLOR_CYAN)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                theme::muted_style()
            };
            let sep = if i > 0 { " " } else { "" };
            vec![Span::raw(sep), Span::styled(*label, style)]
        })
        .collect();

    let tab_line = Line::from(tab_spans);
    let tab_paragraph = Paragraph::new(tab_line).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(theme::muted_style()),
    );
    frame.render_widget(tab_paragraph, chunks[0]);

    // Content area
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::muted_style());

    let content_text = match app.sidebar_state.active_tab {
        SidebarTab::Files => {
            if app.sidebar_state.file_tree.is_empty() {
                "  (no files loaded)".to_string()
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
        SidebarTab::Sessions => "  (no sessions)".to_string(),
        SidebarTab::Skills => "  (no skills loaded)".to_string(),
    };

    let paragraph = Paragraph::new(content_text)
        .block(content_block)
        .style(Style::default().fg(theme::COLOR_GRAY));

    frame.render_widget(paragraph, chunks[1]);
}
