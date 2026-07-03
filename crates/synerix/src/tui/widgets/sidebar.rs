//! Sidebar widget — file tree / session list / skills panel.
//!
//! Professional design with gradient active tab, refined file tree,
//! and smooth visual transitions.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::SidebarTab;
use crate::tui::theme;
use crate::tui::widgets::primitives::RenderContext;

// ── Icons ─────────────────────────────────────────────────────────────────

const ICON_DIR: &str = "◆";       // Directory
const ICON_FILE: &str = "◇";      // File
const ICON_SESSIONS: &str = "◉";  // Sessions
const ICON_SKILLS: &str = "✦";    // Skills
const ICON_EMPTY_DIR: &str = "○"; // Empty directory

// ── Static tab labels (avoid per-frame allocation) ─────────────────────────

const TAB_FILES: &str = "◆ Files";
const TAB_SESSIONS: &str = "◉ Sessions";
const TAB_SKILLS: &str = "✦ Skills";

// ── Tree connectors ────────────────────────────────────────────────────────

const TEE: &str = "├── ";
const ELBOW: &str = "└── ";
const PIPE: &str = "│   ";

/// Render sidebar panel with tab bar and content area.
pub fn render(area: Rect, frame: &mut Frame, ctx: &RenderContext) {
    let p = ctx.palette;

    // Split area into tab bar (1 row) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // ── Tab bar ─────────────────────────────────────────────────────────
    let tabs = [
        (TAB_FILES, SidebarTab::Files),
        (TAB_SESSIONS, SidebarTab::Sessions),
        (TAB_SKILLS, SidebarTab::Skills),
    ];

    // Calculate tab positions for targeted underline
    let mut tab_x = chunks[0].x;
    let mut active_underline: Option<(u16, u16)> = None;

    let mut tab_spans: Vec<Span> = Vec::with_capacity(6);
    for (i, (label, tab)) in tabs.iter().enumerate() {
        let is_active = ctx.sidebar_tab == *tab;
        let style = if is_active {
            Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD)
                .bg(p.highlight_bg)
        } else {
            Style::default().fg(p.muted_fg)
        };
        if i > 0 {
            tab_spans.push(Span::raw(" "));
        }
        let tab_width = label.chars().count() as u16;
        if is_active {
            active_underline = Some((tab_x, tab_width));
        }
        tab_x += tab_width + if i > 0 { 1 } else { 0 };
        tab_spans.push(Span::styled(*label, style));
    }

    let tab_line = Line::from(tab_spans);
    let tab_paragraph = Paragraph::new(tab_line).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_type(theme::BORDER_TYPE)
            .border_style(Style::default().fg(p.border)),
    );
    frame.render_widget(tab_paragraph, chunks[0]);

    // Active tab underline — targeted to tab width only
    if area.height > 1 {
        if let Some((ux, uw)) = active_underline {
            let underline_y = chunks[0].y + chunks[0].height;
            if underline_y < area.y + area.height && uw > 0 {
                let underline_rect = Rect::new(ux, underline_y, uw, 1);
                let underline = Line::from(Span::styled(
                    " ".repeat(uw as usize),
                    Style::default().bg(p.accent),
                ));
                frame.render_widget(Paragraph::new(underline), underline_rect);
            }
        }
    }

    // ── Content area ────────────────────────────────────────────────────
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme::BORDER_TYPE)
        .border_style(Style::default().fg(p.border));

    let content_text = match ctx.sidebar_tab {
        SidebarTab::Files => render_files_tab(ctx),
        SidebarTab::Sessions => format!("  {ICON_SESSIONS} (no sessions)"),
        SidebarTab::Skills => format!("  {ICON_SKILLS} (no skills loaded)"),
    };

    let paragraph = Paragraph::new(content_text)
        .block(content_block)
        .style(Style::default().fg(p.foreground));

    frame.render_widget(paragraph, chunks[1]);
}

/// Render the Files tab with tree-style file tree.
fn render_files_tab(ctx: &RenderContext) -> String {
    use std::fmt::Write;
    if ctx.sidebar_file_tree.is_empty() {
        format!("  {ICON_EMPTY_DIR} (no files loaded)")
    } else {
        let scroll = ctx.sidebar_scroll;
        let mut out = String::new();
        for (i, f) in ctx.sidebar_file_tree.iter().skip(scroll).enumerate() {
            let _ = writeln!(out, "{}", tree_entry(f, i));
        }
        out
    }
}

/// Render a single tree entry with connectors.
fn tree_entry(f: &crate::app::FileEntry, idx: usize) -> String {
    let icon = if f.is_dir { ICON_DIR } else { ICON_FILE };
    let depth = f.depth.saturating_sub(1);
    let indent = if depth > 0 { PIPE } else { "" };
    let connector = if idx == 0 { ELBOW } else { TEE };
    format!("{indent}{connector}{icon}{}", f.name)
}
