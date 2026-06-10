//! Synerix — AI Coding Terminal
//!
//! A high-performance, single-process TUI application that fuses
//! Claude Code's interaction model, Codex CLI's sandbox mechanism,
//! and OpenCode's extensible architecture.

// #![allow(dead_code, unused_imports, unused_variables)] — 已移除，逐模块清理中

mod agent;
mod app;
mod coding_modes;
mod config;
mod error;
mod llm;
mod mcp;
mod project;
mod sandbox;
mod session;
mod skills;
mod slash;
mod telemetry;
mod token_estimator;
mod tools;
mod tui;
mod util;

use config::Settings;
use error::AppError;
use telemetry::{StartupMetrics, StartupTimer};

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
    let result = app::run(settings, metrics).await;

    match &result {
        Ok(()) => tracing::info!("Synerix exited normally"),
        Err(e) => tracing::error!("Synerix exited with error: {}", e),
    }

    result
}
