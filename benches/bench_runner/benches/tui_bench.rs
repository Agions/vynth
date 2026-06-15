use criterion::{black_box, criterion_group, criterion_main, Criterion};
use synerix::tui::diff_renderer::{parse_diff, render_diff, DiffViewMode};
use synerix::tui::widgets::status_bar::format_tokens;

/// Sample unified diff for benchmarking
const SAMPLE_DIFF: &str = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@
 fn hello() {
     println!("Hello");
+    println!("World");
+    println!("from the new line");
 }

@@ -20,12 +22,14 @@
 fn goodbye() {
-    println!("Goodbye");
+    println!("See you later");
     println!("Old code here");
+    println!("More code");
+    println!("Even more");
 }
"#;

fn bench_parse_diff(c: &mut Criterion) {
    c.bench_function("diff/parse_diff_small", |b| {
        b.iter(|| parse_diff(black_box(SAMPLE_DIFF)))
    });
}

fn bench_render_unified(c: &mut Criterion) {
    c.bench_function("diff/render_unified_small", |b| {
        b.iter(|| render_diff(black_box(SAMPLE_DIFF), DiffViewMode::Unified, 0))
    });
}

fn bench_render_side_by_side(c: &mut Criterion) {
    c.bench_function("diff/render_side_by_side_small", |b| {
        b.iter(|| render_diff(black_box(SAMPLE_DIFF), DiffViewMode::SideBySide, 40))
    });
}

fn bench_format_tokens(c: &mut Criterion) {
    c.bench_function("tui/format_tokens", |b| {
        b.iter(|| {
            black_box(format_tokens(black_box(0)));
            black_box(format_tokens(black_box(999)));
            black_box(format_tokens(black_box(1_234)));
            black_box(format_tokens(black_box(12_345)));
            black_box(format_tokens(black_box(128_000)));
            black_box(format_tokens(black_box(1_500_000)));
        })
    });
}

criterion_group!(
    tui_benches,
    bench_parse_diff,
    bench_render_unified,
    bench_render_side_by_side,
    bench_format_tokens
);
criterion_main!(tui_benches);
