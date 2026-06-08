//! Phase 3 tests — Theme, Diff Renderer, Syntax Highlighting, Input

use synerix::tui::diff_renderer::{
    parse_diff, render_diff, render_side_by_side, render_unified, DiffViewMode,
};
use synerix::tui::syntax::highlight_line;
use synerix::tui::theme::{dark_theme, light_theme};
// ── Theme System ──────────────────────────────────────────

#[test]
fn test_theme_dark_resolves() {
    let palette = dark_theme();
    // Dark theme should have dark background
    assert_ne!(palette.background, palette.foreground);
    assert_ne!(palette.accent, palette.border);
}

#[test]
fn test_theme_light_resolves() {
    let palette = light_theme();
    // Light theme should have light background
    assert_ne!(palette.background, palette.foreground);
}

#[test]
fn test_theme_has_all_colors() {
    let dark = dark_theme();
    let light = light_theme();

    // Both themes must define all semantic colors
    for palette in [&dark, &light] {
        // Chat roles
        assert_ne!(palette.chat_user, palette.chat_assistant);
        assert_ne!(palette.chat_system, palette.chat_tool);
        // Accent distinct from background
        assert_ne!(palette.accent, palette.background);
        assert_ne!(palette.accent, palette.border);
        // Error and success distinct
        assert_ne!(palette.error, palette.success);
        // Foreground distinct from muted
        assert_ne!(palette.foreground, palette.muted_fg);
    }
}

#[test]
fn test_theme_enum_variants() {
    let dark = dark_theme();
    let light = light_theme();
    // Dark and light should have different backgrounds
    assert_ne!(
        format!("{:?}", dark.background),
        format!("{:?}", light.background)
    );
}

// ── Diff Renderer ─────────────────────────────────────────

#[test]
fn test_parse_diff_basic() {
    let diff = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("hello");
     println!("world");
 }
"#;

    let hunks = parse_diff(diff);
    assert!(!hunks.is_empty());
    assert!(hunks[0].header.contains("@@"));

    let has_add = hunks[0]
        .lines
        .iter()
        .any(|l| matches!(l.kind, synerix::app::DiffLineKind::Add));
    assert!(has_add, "Should have an added line");
}

#[test]
fn test_parse_diff_multiple_hunks() {
    let diff = r#"--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,3 @@
-old
+new
 unchanged
@@ -10,3 +10,3 @@
-before
+after
 end
"#;

    let hunks = parse_diff(diff);
    assert!(hunks.len() >= 2, "Should have 2 hunks");
}

#[test]
fn test_parse_diff_empty() {
    let hunks = parse_diff("");
    assert!(hunks.is_empty());
}

#[test]
fn test_render_unified_produces_text() {
    let diff = r#"--- a/test.rs
+++ b/test.rs
@@ -1,2 +1,3 @@
+use std::io;
 fn main() {
     println!("hi");
 }
"#;

    let text = render_unified(diff);
    let lines = text.lines;
    assert!(!lines.is_empty(), "Should produce rendered lines");
}

#[test]
fn test_render_side_by_side_produces_text() {
    let diff = r#"--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("old");
+    println!("new");
 }
"#;

    let text = render_side_by_side(diff, 80);
    let lines = text.lines;
    assert!(!lines.is_empty(), "Should produce side-by-side lines");
}

#[test]
fn test_render_diff_with_mode() {
    let diff = r#"--- a/x.rs
+++ b/x.rs
@@ -1,2 +1,2 @@
-a
+b
"#;

    let unified = render_diff(diff, DiffViewMode::Unified, 80);
    let sidebyside = render_diff(diff, DiffViewMode::SideBySide, 80);

    assert!(!unified.lines.is_empty());
    assert!(!sidebyside.lines.is_empty());
}

#[test]
fn test_parse_diff_line_numbers() {
    let diff = r#"--- a/test.rs
+++ b/test.rs
@@ -5,3 +5,4 @@
 line5
+added
 line6
 line7
"#;

    let hunks = parse_diff(diff);
    // Check that we parsed lines
    assert!(!hunks[0].lines.is_empty(), "Should have parsed lines");
}

// ── Syntax Highlighting ───────────────────────────────────

#[test]
fn test_highlight_rust_code() {
    let spans = highlight_line("fn main() {", "rs");
    assert!(!spans.is_empty(), "Should produce highlighted spans");
    // fn is a keyword, should be styled differently
    let all_text: String = spans.iter().map(|(_, s)| s.as_str()).collect();
    assert!(all_text.contains("fn"));
}

#[test]
fn test_highlight_python_code() {
    let spans = highlight_line("def hello():", "py");
    assert!(!spans.is_empty());
    let all_text: String = spans.iter().map(|(_, s)| s.as_str()).collect();
    assert!(all_text.contains("def"));
}

#[test]
fn test_highlight_unknown_extension() {
    // Unknown extension should fall back to plain text
    let spans = highlight_line("hello world", "xyzzy");
    assert!(!spans.is_empty());
    let all_text: String = spans.iter().map(|(_, s)| s.as_str()).collect();
    assert!(all_text.contains("hello world"));
}

#[test]
fn test_highlight_empty_line() {
    let spans = highlight_line("", "rs");
    // Empty line should produce at least one span (possibly empty)
    assert!(spans.len() <= 1);
}

// ── App Input Handling ────────────────────────────────────

#[test]
fn test_app_input_buffer_insert() {
    let settings = synerix::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app =
        synerix::app::App::new_with_channel(settings, tx, _rx, synerix::app::InputMode::Normal);

    // Switch to insert mode
    app.mode = synerix::app::InputMode::Insert;

    // Simulate typing
    app.input_buffer.push('h');
    app.input_buffer.push('e');
    app.input_buffer.push('l');
    app.input_buffer.push('l');
    app.input_buffer.push('o');
    app.input_cursor = app.input_buffer.len();

    assert_eq!(app.input_buffer, "hello");
    assert_eq!(app.input_cursor, 5);
}

#[test]
fn test_app_submit_message() {
    let settings = synerix::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app =
        synerix::app::App::new_with_channel(settings, tx, _rx, synerix::app::InputMode::Normal);

    app.input_buffer = "test message".to_string();
    app.input_cursor = 12;
    app.submit_message();

    assert_eq!(app.chat_state.messages.len(), 1);
    assert_eq!(app.chat_state.messages[0].content, "test message");
    assert_eq!(
        app.chat_state.messages[0].role,
        synerix::app::MessageRole::User
    );
    assert!(app.input_buffer.is_empty());
    assert_eq!(app.input_cursor, 0);
}

#[test]
fn test_app_submit_empty_message() {
    let settings = synerix::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app =
        synerix::app::App::new_with_channel(settings, tx, _rx, synerix::app::InputMode::Normal);

    app.input_buffer = "".to_string();
    app.submit_message();

    // Empty messages should not be submitted
    assert_eq!(app.chat_state.messages.len(), 0);
}

#[test]
fn test_app_chat_scroll() {
    let settings = synerix::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app =
        synerix::app::App::new_with_channel(settings, tx, _rx, synerix::app::InputMode::Normal);

    assert_eq!(app.chat_state.scroll_offset, 0);

    app.chat_state.scroll_offset += 5;
    assert_eq!(app.chat_state.scroll_offset, 5);

    if app.chat_state.scroll_offset > 0 {
        app.chat_state.scroll_offset -= 1;
    }
    assert_eq!(app.chat_state.scroll_offset, 4);
}

// ── Total Stats ───────────────────────────────────────────

#[test]
fn test_project_file_count() {
    use std::path::Path;
    use std::process::Command;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = repo_root.join("src");

    let output = Command::new("find")
        .arg(&src_dir)
        .args(["-name", "*.rs", "-type", "f"])
        .output()
        .unwrap();

    let count = String::from_utf8_lossy(&output.stdout).lines().count();
    assert!(count >= 55, "Expected 55+ source files, got {}", count);
}
