//! Syncode — AI Pair Programming Terminal
//!
//! A high-performance, single-process TUI application that fuses
//! Claude Code's interaction model, Codex CLI's sandbox mechanism,
//! and OpenCode's extensible architecture.

#![allow(dead_code, unused_imports, unused_variables)]

mod app;
mod error;
mod config;
mod tui;
mod session;
mod agent;
mod llm;
mod tools;
mod skills;
mod mcp;
mod sandbox;

use config::Settings;
use error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Syncode v{} starting up", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let settings = Settings::load()?;
    tracing::info!("Configuration loaded");

    // Run the application
    let result = app::run(settings).await;

    match &result {
        Ok(()) => tracing::info!("Syncode exited normally"),
        Err(e) => tracing::error!("Syncode exited with error: {}", e),
    }

    result
}
