//! Syntax highlighting via syntect
//!
//! Provides line-level syntax highlighting with cached SyntaxSet/ThemeSet.

use ratatui::style::{Color, Style};
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached SyntaxSet — loaded once and reused across the process lifetime.
fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Cached ThemeSet with the base16-ocean.dark theme pre-selected.
fn theme() -> &'static syntect::highlighting::Theme {
    static THEME: OnceLock<syntect::highlighting::Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-ocean.dark"].clone()
    })
}

/// Highlight a single line of code.
///
/// Returns a vector of `(Style, String)` pairs suitable for building ratatui
/// `Span`s.  `extension` is the file extension (e.g. `"rs"`, `"py"`, `"js"`)
/// used to pick the right syntax definition.  If the extension is not
/// recognized the line is returned as plain text.
pub fn highlight_line(line: &str, extension: &str) -> Vec<(Style, String)> {
    let ss = syntax_set();
    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, theme());

    // We highlight the single line (add a trailing newline so syntect is happy)
    let input = if line.ends_with('\n') {
        line.to_string()
    } else {
        format!("{}\n", line)
    };

    let mut result = Vec::new();
    for single_line in LinesWithEndings::from(&input) {
        match h.highlight_line(single_line, ss) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    result.push((syntect_to_ratatui_style(style), text.to_string()));
                }
            }
            Err(_) => {
                // On any highlight error fall back to plain text
                result.push((Style::default(), single_line.to_string()));
            }
        }
    }

    if result.is_empty() {
        result.push((Style::default(), line.to_string()));
    }

    result
}

/// Convert a syntect `Style` into a ratatui `Style`.
fn syntect_to_ratatui_style(s: SynStyle) -> Style {
    Style::default().fg(Color::Rgb(s.foreground.r, s.foreground.g, s.foreground.b))
}
