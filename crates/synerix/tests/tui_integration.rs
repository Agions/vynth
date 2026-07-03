//! Integration tests for TUI rendering components.
//!
//! These tests verify the pure rendering functions produce correct output
//! without requiring a terminal or ratatui rendering context.

use synerix::tui::diff_renderer::{parse_diff, render_diff, DiffViewMode};
use synerix::tui::theme::{read_palette, init_theme};

// ── Diff Parsing ───────────────────────────────────────────

const SAMPLE_DIFF: &str = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@
 fn hello() {
     println!("Hello");
+    println!("World");
+    println!("from the new line");
 }
"#;

#[test]
fn test_parse_diff_basic() {
    let hunks = parse_diff(SAMPLE_DIFF);
    assert_eq!(hunks.len(), 1, "Should parse one hunk");
    assert_eq!(
        hunks[0].lines.len(),
        5,
        "Hunk should have 5 lines (3 context + 2 added)"
    );
}

#[test]
fn test_parse_diff_counts_adds_and_removes() {
    let hunks = parse_diff(SAMPLE_DIFF);
    let adds = hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == synerix::app::DiffLineKind::Add)
        .count();
    let removes = hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == synerix::app::DiffLineKind::Remove)
        .count();
    let context = hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == synerix::app::DiffLineKind::Context)
        .count();
    assert_eq!(adds, 2, "Should have 2 added lines");
    assert_eq!(removes, 0, "Should have 0 removed lines");
    assert_eq!(context, 3, "Should have 3 context lines");
}

#[test]
fn test_parse_diff_hunk_header() {
    let hunks = parse_diff(SAMPLE_DIFF);
    assert!(hunks[0].header.contains("@@"), "Header should contain @@");
}

#[test]
fn test_parse_diff_empty() {
    let hunks = parse_diff("");
    assert!(hunks.is_empty(), "Empty diff should produce no hunks");
}

#[test]
fn test_parse_diff_invalid_header() {
    let diff = "some random text\nthat is not a diff\n";
    let hunks = parse_diff(diff);
    assert!(hunks.is_empty(), "Invalid diff should produce no hunks");
}

#[test]
fn test_parse_diff_remove_lines() {
    let diff = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,0 @@
-fn old_function() {
-    println!("remove me");
-}
"#;
    let hunks = parse_diff(diff);
    assert_eq!(hunks.len(), 1);
    let removes = hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == synerix::app::DiffLineKind::Remove)
        .count();
    assert_eq!(removes, 3, "Should have 3 removed lines");
}

// ── Diff Rendering ─────────────────────────────────────────

#[test]
fn test_render_unified_returns_text() {
    init_theme(true);
    let text = render_diff(SAMPLE_DIFF, DiffViewMode::Unified, 0, &read_palette());
    let lines: Vec<_> = text.lines.iter().collect();
    assert!(!lines.is_empty(), "Unified render should produce lines");
    // First line should be the hunk header
    let first = format!("{:?}", lines[0]);
    assert!(first.contains("@@"), "First line should contain @@ header");
}

#[test]
fn test_render_unified_empty() {
    init_theme(true);
    let text = render_diff("", DiffViewMode::Unified, 0, &read_palette());
    let lines: Vec<_> = text.lines.iter().collect();
    assert!(
        !lines.is_empty(),
        "Empty diff should still produce a placeholder line"
    );
    let rendered = format!("{:?}", lines[0]);
    assert!(
        rendered.contains("no diff"),
        "Empty diff should show placeholder"
    );
}

#[test]
fn test_render_side_by_side_returns_text() {
    init_theme(true);
    let text = render_diff(SAMPLE_DIFF, DiffViewMode::SideBySide, 40, &read_palette());
    let lines: Vec<_> = text.lines.iter().collect();
    assert!(
        !lines.is_empty(),
        "Side-by-side render should produce lines"
    );
}

// End of TUI integration tests
