//! Synerix — AI Coding Terminal
//!
//! A high-performance, single-process TUI application that fuses
//! Claude Code's interaction model, Codex CLI's sandbox mechanism,
//! and OpenCode's extensible architecture.

use synerix::config::Settings;
use synerix::error::AppError;
use synerix::telemetry::{StartupMetrics, StartupTimer};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let mut timer = StartupTimer::new();

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Synerix v{} starting up", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let settings = Settings::load()?;
    let config_load_ms = timer.mark();
    tracing::info!("Configuration loaded ({}ms)", config_load_ms);

    // TUI init is done inside app::run (after App is constructed)
    let total_ms = timer.total_elapsed_ms();

    let metrics = StartupMetrics {
        config_load_ms,
        tui_init_ms: 0, // will be set during app::run when TUI is initialized
        total_ms,
    };

    metrics.log();

    if cfg!(feature = "startup-bench") {
        metrics.eprint();
    }

    // Run the application
    let result = synerix::app::run(settings, metrics).await;

    match &result {
        Ok(()) => tracing::info!("Synerix exited normally"),
        Err(e) => tracing::error!("Synerix exited with error: {}", e),
    }

    result
}
