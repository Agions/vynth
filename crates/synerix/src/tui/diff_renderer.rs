//! Diff renderer — parses unified diffs and produces syntax-highlighted ratatui `Text`.
//!
//! Supports two view modes:
//! - **Unified**: standard unified-diff layout (lines prefixed with `+`, `-`, ` `)
//! - **Side-by-side**: old and new code displayed in two columns

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use crate::app::{DiffHunk, DiffLine, DiffLineKind};
use crate::tui::syntax;
use crate::tui::theme;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// View mode for diff rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
}

/// A richer diff line used by the renderer — tracks line numbers.
#[derive(Debug, Clone)]
pub struct RenderDiffLine {
    pub content: String,
    pub line_type: DiffLineKind,
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse a raw unified-diff string into [`DiffHunk`]s with line numbers.
pub fn parse_diff(raw: &str) -> Vec<DiffHunk> {
    // Pre-allocate with reasonable estimates to reduce re-allocations.
    let estimated_hunks = raw.matches("\n@@").count().max(1);
    let mut hunks: Vec<DiffHunk> = Vec::with_capacity(estimated_hunks);
    let mut current_header = String::with_capacity(128);
    let mut current_lines: Vec<DiffLine> = Vec::with_capacity(32);
    let mut _old_line: usize = 0;
    let mut _new_line: usize = 0;

    for line in raw.lines() {
        if line.starts_with("@@") {
            // Flush previous hunk if any
            if !current_header.is_empty() || !current_lines.is_empty() {
                hunks.push(DiffHunk {
                    header: std::mem::take(&mut current_header),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            // Parse the @@ header for starting line numbers
            if let Some((old, new)) = parse_hunk_header(line) {
                _old_line = old;
                _new_line = new;
            }
            current_header = line.to_string();
        } else if line.starts_with('+') && !line.starts_with("+++") {
            current_lines.push(DiffLine {
                kind: DiffLineKind::Add,
                content: line[1..].to_string(),
            });
        } else if line.starts_with('-') && !line.starts_with("---") {
            current_lines.push(DiffLine {
                kind: DiffLineKind::Remove,
                content: line[1..].to_string(),
            });
        } else if let Some(stripped) = line.strip_prefix(' ') {
            current_lines.push(DiffLine {
                kind: DiffLineKind::Context,
                content: stripped.to_string(),
            });
        } else if line.starts_with("\\") {
            // "\ No newline at end of file" — skip
        }
        // Other lines (---, +++, diff --git, etc.) are skipped
    }

    // Flush last hunk
    if !current_header.is_empty() || !current_lines.is_empty() {
        hunks.push(DiffHunk {
            header: current_header,
            lines: current_lines,
        });
    }

    hunks
}

/// Parse `@@ -old_start,old_count +new_start,new_count @@` header.
fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    // e.g. "@@ -10,6 +10,8 @@"
    let rest = line.strip_prefix("@@ ")?;
    let parts: Vec<&str> = rest.split(" @@").collect();
    let ranges = parts.first()?;
    let mut old_start = 0usize;
    let mut new_start = 0usize;
    for token in ranges.split_whitespace() {
        if let Some(n) = token.strip_prefix("-") {
            old_start = n.split(',').next()?.parse().ok()?;
        } else if let Some(n) = token.strip_prefix("+") {
            new_start = n.split(',').next()?.parse().ok()?;
        }
    }
    Some((old_start, new_start))
}

/// Parse starting line numbers from a hunk header string.
///
/// Returns `(old_line, new_line)` — 1-based line numbers, or `(0, 0)` if unparseable.
fn parse_hunk_start_lines(header: &str) -> (usize, usize) {
    // Strip the @@ wrapper and parse the ranges
    let rest = header.strip_prefix("@@ ").unwrap_or(header);
    let parts: Vec<&str> = rest.split(" @@").collect();
    let ranges = parts.first().unwrap_or(&"");
    let mut old_start = 0usize;
    let mut new_start = 0usize;
    for token in ranges.split_whitespace() {
        if let Some(n) = token.strip_prefix("-") {
            old_start = n.split(',').next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(n) = token.strip_prefix("+") {
            new_start = n.split(',').next().and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }
    (old_start, new_start)
}

// ---------------------------------------------------------------------------
// Rendering — unified view
// ---------------------------------------------------------------------------

/// Guess a file extension from the diff content (first `+++ b/...` line).
fn guess_extension(diff_text: &str) -> String {
    for line in diff_text.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            if let Some(ext) = path.rsplit('.').next() {
                return ext.to_string();
            }
        }
    }
    String::new()
}

/// Render a raw unified diff string as a ratatui `Text` (unified view).
///
/// Each line is syntax-highlighted via syntect and tinted with the
/// conventional diff colours (green for added, red for removed, etc.).
pub fn render_unified(diff_text: &str, p: &theme::ColorPalette) -> Text<'static> {
    let ext = guess_extension(diff_text);
    let hunks = parse_diff(diff_text);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for hunk in &hunks {
        // Hunk header
        lines.push(Line::from(Span::styled(
            hunk.header.clone(),
            Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),
        )));

        // Parse starting line numbers from @@ header
        let (mut old_no, mut new_no) = parse_hunk_start_lines(&hunk.header);

        for dl in &hunk.lines {
            let (prefix, bg, fg) = match dl.kind {
                DiffLineKind::Add => (
                    "+",
                    p.diff_add_bg,
                    p.diff_add_fg,
                ),
                DiffLineKind::Remove => (
                    "-",
                    p.diff_remove_bg,
                    p.diff_remove_fg,
                ),
                DiffLineKind::Context => (" ", Color::Reset, p.comment),
            };

            // Format line number
            let line_no_str = match dl.kind {
                DiffLineKind::Add => format!("{:>4}", new_no),
                DiffLineKind::Remove => format!("{:>4}", old_no),
                DiffLineKind::Context => format!("{:>4}", old_no),
            };

            // Advance line numbers
            match dl.kind {
                DiffLineKind::Add => { new_no += 1; }
                DiffLineKind::Remove => { old_no += 1; }
                DiffLineKind::Context => {
                    old_no += 1;
                    new_no += 1;
                }
            }

            // Syntax-highlight the code content
            let highlighted = syntax::highlight_line(&dl.content, &ext);
            let mut spans: Vec<Span<'static>> = Vec::new();

            // Line number gutter (left side)
            let ln_color = match dl.kind {
                DiffLineKind::Add => p.diff_add_fg,
                DiffLineKind::Remove => p.diff_remove_fg,
                DiffLineKind::Context => p.comment,
            };
            spans.push(Span::styled(
                line_no_str,
                Style::default().fg(ln_color).bg(bg),
            ));
            spans.push(Span::styled(
                " ",
                Style::default().fg(ln_color).bg(bg),
            ));

            // Prefix character
            spans.push(Span::styled(
                prefix.to_string(),
                Style::default().fg(fg).bg(bg),
            ));

            // Highlighted content spans
            for (style, text) in highlighted {
                spans.push(Span::styled(text, style.fg(fg).bg(bg)));
            }

            lines.push(Line::from(spans));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no diff content)",
            theme::muted_style(),
        )));
    }

    Text::from(lines)
}

// ---------------------------------------------------------------------------
// Rendering — side-by-side view
// ---------------------------------------------------------------------------

/// Render a raw unified diff string as a ratatui `Text` (side-by-side view).
///
/// Old lines appear on the left, new lines on the right, separated by a
/// `│` gutter.  The column width is `col_width` characters per side.
pub fn render_side_by_side(diff_text: &str, col_width: usize, p: &theme::ColorPalette) -> Text<'static> {
    let ext = guess_extension(diff_text);
    let hunks = parse_diff(diff_text);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for hunk in &hunks {
        // Hunk header spans full width
        lines.push(Line::from(Span::styled(
            hunk.header.clone(),
            Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),
        )));

        // Pair up removed / added / context lines
        let mut idx = 0;
        while idx < hunk.lines.len() {
            let dl = &hunk.lines[idx];

            match dl.kind {
                DiffLineKind::Context => {
                    // Same content on both sides
                    let left = format_side(&dl.content, col_width);
                    let right = format_side(&dl.content, col_width);
                    let spans = vec![
                        Span::styled(left, Style::default().fg(p.comment)),
                        Span::styled("│", theme::muted_style()),
                        Span::styled(right, Style::default().fg(p.comment)),
                    ];
                    lines.push(Line::from(spans));
                    idx += 1;
                }
                DiffLineKind::Remove => {
                    // Gather consecutive removes
                    let mut removes = Vec::new();
                    while idx < hunk.lines.len() && hunk.lines[idx].kind == DiffLineKind::Remove {
                        removes.push(&hunk.lines[idx]);
                        idx += 1;
                    }
                    // Gather consecutive adds that follow
                    let mut adds = Vec::new();
                    while idx < hunk.lines.len() && hunk.lines[idx].kind == DiffLineKind::Add {
                        adds.push(&hunk.lines[idx]);
                        idx += 1;
                    }
                    let max = removes.len().max(adds.len());
                    for i in 0..max {
                        let mut spans = Vec::new();
                        if i < removes.len() {
                            let highlighted = syntax::highlight_line(&removes[i].content, &ext);
                            let text: String =
                                highlighted.iter().map(|(_, s)| s.as_str()).collect();
                            let left = format_side(&text, col_width);
                            spans.push(Span::styled(
                                left,
                                Style::default()
                                    .fg(p.diff_remove_fg)
                                    .bg(p.diff_remove_bg),
                            ));
                        } else {
                            spans.push(Span::raw(" ".repeat(col_width)));
                        }
                        spans.push(Span::styled("│", theme::muted_style()));
                        if i < adds.len() {
                            let highlighted = syntax::highlight_line(&adds[i].content, &ext);
                            let text: String =
                                highlighted.iter().map(|(_, s)| s.as_str()).collect();
                            let right = format_side(&text, col_width);
                            spans.push(Span::styled(
                                right,
                                Style::default()
                                    .fg(p.diff_add_fg)
                                    .bg(p.diff_add_bg),
                            ));
                        } else {
                            spans.push(Span::raw(" ".repeat(col_width)));
                        }
                        lines.push(Line::from(spans));
                    }
                }
                DiffLineKind::Add => {
                    // Orphaned add (no preceding remove)
                    let highlighted = syntax::highlight_line(&dl.content, &ext);
                    let text: String = highlighted.iter().map(|(_, s)| s.as_str()).collect();
                    let right = format_side(&text, col_width);
                    let spans = vec![
                        Span::raw(" ".repeat(col_width)),
                        Span::styled("│", theme::muted_style()),
                        Span::styled(
                            right,
                            Style::default()
                                .fg(p.diff_add_fg)
                                .bg(p.diff_add_bg),
                        ),
                    ];
                    lines.push(Line::from(spans));
                    idx += 1;
                }
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no diff content)",
            theme::muted_style(),
        )));
    }

    Text::from(lines)
}

/// Truncate or pad `text` to exactly `width` chars (single allocation).
fn format_side(text: &str, width: usize) -> String {
    let mut s = String::with_capacity(width);
    for ch in text.chars().take(width) {
        s.push(ch);
    }
    for _ in s.chars().count()..width {
        s.push(' ');
    }
    s
}

// ---------------------------------------------------------------------------
// High-level entry point
// ---------------------------------------------------------------------------

/// Render `diff_text` using the given view mode.
///
/// For side-by-side mode `col_width` controls the width of each column
/// (defaults to 40 when the caller passes 0).
pub fn render_diff(diff_text: &str, mode: DiffViewMode, col_width: usize, p: &theme::ColorPalette) -> Text<'static> {
    match mode {
        DiffViewMode::Unified => render_unified(diff_text, p),
        DiffViewMode::SideBySide => {
            let w = if col_width == 0 { 40 } else { col_width };
            render_side_by_side(diff_text, w, p)
        }
    }
}
