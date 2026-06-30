//! Text sanitization pipeline for terminal rendering.
//!
//! Centralizes the HTML/Markdown cleaning logic that was previously split
//! between `tui::widgets::chat_area` (`clean_display_text`) and
//! `slash::common` (`clean_terminal_text`). Both produced plain text fit for a
//! TUI; consolidating them here is the first step toward a single configurable
//! pipeline (see roadmap Phase 3.1).
//!
//! Public entry points:
//! - [`render_plain_text`] — full HTML + Markdown + table normalization, used
//!   for streamed assistant/chat content.
//! - [`sanitize_terminal_text`] — lightweight marker/emoji stripping, used for
//!   short system messages emitted by slash commands.

/// Render rich assistant text as terminal-friendly plain text.
///
/// Pipeline: strip HTML tags → decode HTML entities → strip Markdown markers →
/// convert Markdown tables to aligned box-drawing tables.
pub fn render_plain_text(input: &str) -> String {
    let html_text = strip_html_tags(input);
    let markdown_clean = decode_html_entities(&html_text);
    convert_markdown_tables(&strip_markdown_markers(&markdown_clean))
}

/// Strip Markdown emphasis markers and replace emoji with ASCII labels.
///
/// Used for the short system messages that slash commands push into the chat
/// transcript, where the full Markdown pipeline is unnecessary.
pub fn sanitize_terminal_text(text: &str) -> String {
    text.replace("**", "")
        .replace('`', "")
        .replace("✅", "OK")
        .replace("❌", "ERROR")
        .replace("💡", "TIP")
        .replace("📋", "")
        .replace("📖", "")
        .replace("⚙️", "")
        .replace("📂", "")
        .replace("🎯", "")
        .replace("📦", "")
        .replace("🔀", "")
        .replace("🤖", "")
        .replace("🚀", "RUN")
        .replace("🗑️", "REMOVED")
        .replace("💾", "SAVED")
        .replace("🔄", "RESET")
        .replace("🧠", "")
        .replace("⚡", "")
        .replace("💬", "")
        .replace("🔧", "")
        .replace("🎵", "")
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag = String::new();
            let mut closed = false;
            for tag_ch in chars.by_ref() {
                if tag_ch == '>' {
                    closed = true;
                    break;
                }
                tag.push(tag_ch);
            }

            if !closed {
                output.push('<');
                output.push_str(&tag);
                break;
            }

            let tag_name = tag
                .trim()
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('/');
            match tag_name.to_ascii_lowercase().as_str() {
                "br" => output.push('\n'),
                "p" | "div" | "section" | "article" | "li" | "ul" | "ol" | "pre"
                    if !output.ends_with('\n') && !output.is_empty() =>
                {
                    output.push('\n');
                }
                _ => {}
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn strip_markdown_markers(input: &str) -> String {
    input
        .lines()
        .map(sanitize_markdown_line)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn sanitize_markdown_line(line: &str) -> String {
    let mut text = line.trim_start().to_string();
    while text.starts_with('#') {
        text.remove(0);
    }
    text = text.trim_start().to_string();

    for marker in ["```", "**", "__", "`", "~~"] {
        text = text.replace(marker, "");
    }

    text
}

/// Convert Markdown pipe tables to aligned box-drawing tables.
fn convert_markdown_tables(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if looks_like_table_row(lines[i])
            && i + 1 < lines.len()
            && looks_like_table_row(lines[i + 1])
        {
            let (table_lines, next_i) = extract_table_block(&lines, i);
            let formatted = format_table_block(table_lines);
            result.extend(formatted.lines().map(|s| s.to_string()));
            i = next_i;
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    result.join("\n")
}

fn looks_like_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|')
}

fn extract_table_block<'a>(lines: &'a [&'a str], start: usize) -> (Vec<&'a str>, usize) {
    let mut end = start;
    while end < lines.len() && looks_like_table_row(lines[end]) {
        end += 1;
    }
    // Don't consume blank separator lines after the table
    (lines[start..end].to_vec(), end)
}

fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed[1..trimmed.len() - 1].contains('-')
}

fn parse_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|s| s.trim().to_string())
        .collect()
}

fn format_table_block(table_lines: Vec<&str>) -> String {
    if table_lines.is_empty() {
        return String::new();
    }

    let header = parse_row(table_lines[0]);
    let data_start = if table_lines.len() > 1 { 2 } else { 1 };
    let mut rows = Vec::new();
    for line in table_lines.iter().skip(data_start) {
        if !is_separator_row(line) {
            rows.push(parse_row(line));
        }
    }

    let num_cols = header.len();
    let mut col_widths = vec![0usize; num_cols];
    for (idx, cell) in header.iter().enumerate() {
        col_widths[idx] = cell.len();
    }
    for row in &rows {
        for (idx, cell) in row.iter().enumerate() {
            if idx < num_cols {
                col_widths[idx] = col_widths[idx].max(cell.len());
            }
        }
    }

    let mut formatted = Vec::new();
    formatted.push(format_row(&header, &col_widths));
    formatted.push(format_separator(&col_widths));
    for row in &rows {
        formatted.push(format_row(row, &col_widths));
    }

    formatted.join("\n")
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    let mut parts = Vec::new();
    for (idx, cell) in cells.iter().enumerate() {
        let w = widths.get(idx).copied().unwrap_or(0);
        parts.push(format!(" {:<w$} ", cell, w = w));
    }
    parts.join("│")
}

fn format_separator(widths: &[usize]) -> String {
    let mut parts = Vec::new();
    for &w in widths {
        parts.push("─".repeat(w + 2));
    }
    parts.join("┼")
}

#[cfg(test)]
mod tests {
    use super::{convert_markdown_tables, render_plain_text, sanitize_terminal_text};

    #[test]
    fn render_plain_text_strips_markdown_markers() {
        assert_eq!(render_plain_text("## **Hello** `world`"), "Hello world");
    }

    #[test]
    fn render_plain_text_converts_basic_html() {
        assert_eq!(
            render_plain_text("<p>Hello<br>world &amp; Synerix</p>"),
            "Hello\nworld & Synerix"
        );
    }

    #[test]
    fn convert_markdown_table_renders_aligned() {
        let input =
            "| Mode | Icon | Description |\n|------|------|-------------|\n| Act | ⚡ | Execute |";
        let output = convert_markdown_tables(input);
        assert!(output.contains("Act"));
        assert!(output.contains("⚡"));
        assert!(output.contains("Execute"));
        assert!(!output.contains("|------|"));
    }

    #[test]
    fn sanitize_terminal_text_strips_markers_and_emoji() {
        assert_eq!(sanitize_terminal_text("**bold** `code` ✅"), "bold code OK");
    }
}
