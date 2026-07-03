//! Lightweight syntax highlighting — replaces syntect to save ~4MB binary size.
//!
//! Handles the most common tokens: keywords, strings, comments, and numbers.
//! Uses simple regex-free scanning for minimal overhead.

use ratatui::style::{Color, Style};

/// Highlight a single line of code into (Style, text) segments.
///
/// Returns a vector of styled segments suitable for building ratatui `Span`s.
/// This is a minimal implementation focused on the most common cases
/// rather than full language parsing.
pub fn highlight_line(line: &str, _extension: &str) -> Vec<(Style, String)> {
    let mut result = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Try to match comments first
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            // Single-line comment
            let comment: String = chars[i..].iter().collect();
            result.push((
                Style::default().fg(Color::Rgb(120, 126, 160)),
                comment,
            ));
            break;
        }

        // Try to match strings
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            i += 1;
            let start = i - 1;
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < chars.len() {
                i += 1; // include closing quote
            }
            let string: String = chars[start..i].iter().collect();
            result.push((
                Style::default().fg(Color::Rgb(152, 195, 127)),
                string,
            ));
            continue;
        }

        // Try to match numbers
        if chars[i].is_ascii_digit() || (chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_' || chars[i] == 'x' || chars[i] == 'b' || chars[i] == 'o') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '+' || chars[i] == '-' || chars[i] == '_') {
                    i += 1;
                }
            }
            let num: String = chars[start..i].iter().collect();
            result.push((
                Style::default().fg(Color::Rgb(209, 154, 102)),
                num,
            ));
            continue;
        }

        // Try to match keywords
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if is_keyword(&word) {
                result.push((
                    Style::default().fg(Color::Rgb(198, 120, 221)).add_modifier(ratatui::style::Modifier::BOLD),
                    word,
                ));
            } else if word == "self" || word == "Self" {
                result.push((
                    Style::default().fg(Color::Rgb(97, 175, 239)),
                    word,
                ));
            } else {
                result.push((Style::default(), word));
            }
            continue;
        }

        // Skip punctuation and whitespace
        result.push((Style::default(), chars[i].to_string()));
        i += 1;
    }

    if result.is_empty() {
        result.push((Style::default(), line.to_string()));
    }

    result
}

/// Check if a word is a Rust keyword.
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let" | "mut" | "const" | "pub" | "struct" | "impl"
            | "use" | "mod" | "return" | "if" | "else" | "for"
            | "while" | "match" | "loop" | "break" | "continue"
            | "true" | "false" | "where" | "type" | "trait" | "enum"
            | "async" | "await" | "move" | "dyn" | "unsafe" | "extern"
            | "crate" | "super" | "in" | "ref" | "static"
    )
}
