//! Application controller — global state machine + event dispatch
//!
//! Split into sub-modules:
//! - `state` — All struct/enum definitions + constructors
//! - `event_loop` — Event loop + input handling + agent events
//! - `actions` — Action execution (the big match block)

mod actions;
mod event_loop;
mod state;

// Re-export all public types so external code sees the same API
pub use state::*;

use crate::config::Settings;
use crate::error::AppError;

/// Run the application
pub async fn run(
    settings: Settings,
    startup_metrics: crate::telemetry::StartupMetrics,
) -> Result<(), AppError> {
    tracing::info!("Initializing application");

    let mut app = App::new(settings);

    // Attach startup metrics to status bar
    app.status_bar.startup_metrics = Some(startup_metrics);

    // Initialize TUI
    let mut terminal = crate::tui::init()?;

    let result = app.run(&mut terminal).await;

    // Restore terminal
    crate::tui::restore(terminal)?;

    result
}
