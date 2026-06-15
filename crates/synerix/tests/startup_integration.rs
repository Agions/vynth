//! Integration tests for application startup and configuration loading.
//!
//! Verifies the initialization sequence produces correct results
//! without actually starting the TUI.

use synerix::config::Settings;
use synerix::telemetry::{StartupMetrics, StartupTimer};

// ── Startup Metrics ────────────────────────────────────────

#[test]
fn test_startup_timer_basic() {
    let mut timer = StartupTimer::new();
    let elapsed = timer.mark();
    assert!(
        elapsed < 100,
        "First mark should be near-instant, got {elapsed}ms"
    );
    let total = timer.total_elapsed_ms();
    assert!(
        total < 100,
        "Total elapsed should also be near-instant, got {total}ms"
    );
}

#[test]
fn test_startup_timer_sequential_marks() {
    let mut timer = StartupTimer::new();
    let _first = timer.mark();
    let _second = timer.mark();
    let total = timer.total_elapsed_ms();
    // Marks should not cause errors; total should be reasonable
    assert!(total >= 0, "Total elapsed should be non-negative");
}

// ── Startup Metrics ────────────────────────────────────────

#[test]
fn test_startup_metrics_log_does_not_panic() {
    let metrics = StartupMetrics {
        config_load_ms: 5,
        tui_init_ms: 10,
        total_ms: 15,
    };
    // These should not panic
    metrics.log();
}

#[test]
fn test_startup_metrics_eprint_does_not_panic() {
    let metrics = StartupMetrics {
        config_load_ms: 5,
        tui_init_ms: 10,
        total_ms: 15,
    };
    // This should not panic even outside the startup-bench feature
    metrics.eprint();
}

// ── Config Loading ─────────────────────────────────────────

#[test]
fn test_settings_load_has_all_sections() {
    let settings = Settings::load().expect("Settings should load from default config");
    // Verify all top-level config sections exist
    let _ = &settings.llm;
    let _ = &settings.ui;
    let _ = &settings.sandbox;
    let _ = &settings.skills_dir;
    let _ = &settings.mcp;
}

#[test]
fn test_settings_llm_section() {
    let settings = Settings::load().unwrap();
    assert!(
        !settings.llm.model.is_empty(),
        "Model name should not be empty"
    );
    assert!(
        settings.llm.context_window >= 4096,
        "Context window should be at least 4K"
    );
    assert!(
        settings.llm.max_output_tokens >= 512,
        "Max output tokens should be at least 512"
    );
    assert!(
        settings.llm.temperature >= 0.0,
        "Temperature should be non-negative"
    );
}

// ── Coding Modes ───────────────────────────────────────────

#[test]
fn test_coding_mode_labels() {
    for mode in &[
        synerix::coding_modes::CodingMode::Plan,
        synerix::coding_modes::CodingMode::Act,
        synerix::coding_modes::CodingMode::Chat,
        synerix::coding_modes::CodingMode::Architect,
        synerix::coding_modes::CodingMode::Vibe,
    ] {
        let label = mode.label();
        assert!(!label.is_empty(), "Coding mode label should not be empty");
        assert!(
            !label.contains('\n'),
            "Coding mode label should be single line"
        );
    }
}
