//! Startup metrics collection and reporting
//!
//! Tracks timing for each major initialization phase and provides
//! logging and status-bar display helpers.

use std::time::Instant;

/// Timing metrics collected during application startup
#[derive(Debug, Clone)]
pub struct StartupMetrics {
    /// Time to load and parse config (ms)
    pub config_load_ms: u64,
    /// Time to initialize TUI terminal (ms)
    pub tui_init_ms: u64,
    /// Time to open SQLite session store (ms)
    pub db_open_ms: u64,
    /// Total wall-clock startup time (ms)
    pub total_ms: u64,
}

impl StartupMetrics {
    /// Log all metrics at INFO level
    pub fn log(&self) {
        tracing::info!(
            config_load_ms = self.config_load_ms,
            tui_init_ms = self.tui_init_ms,
            db_open_ms = self.db_open_ms,
            total_ms = self.total_ms,
            "Startup metrics"
        );
    }

    /// Print metrics to stderr (for `startup_bench` feature flag)
    pub fn eprint(&self) {
        eprintln!(
            "[startup_bench] config_load={}ms  tui_init={}ms  db_open={}ms  total={}ms",
            self.config_load_ms, self.tui_init_ms, self.db_open_ms, self.total_ms
        );
    }

    /// Format a short summary suitable for the status bar
    pub fn status_bar_text(&self) -> String {
        format!("startup: {}ms", self.total_ms)
    }
}

/// A simple stopwatch for measuring sequential startup phases.
///
/// Each call to [`mark`](StartupTimer::mark) returns the milliseconds
/// elapsed since the previous mark (or since creation for the first call).
pub struct StartupTimer {
    start: Instant,
    last_mark: Instant,
}

impl StartupTimer {
    /// Create a new timer, starting the clock immediately.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_mark: now,
        }
    }

    /// End the current phase and return its duration in milliseconds.
    pub fn mark(&mut self) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_mark).as_millis() as u64;
        self.last_mark = now;
        elapsed
    }

    /// Total milliseconds elapsed since the timer was created.
    pub fn total_elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl Default for StartupTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_timer_marks_phases() {
        let mut timer = StartupTimer::new();
        // First mark should be ~0ms
        let phase1 = timer.mark();
        assert!(phase1 < 100, "first phase should be < 100ms, got {phase1}");

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        let phase2 = timer.mark();
        assert!(phase2 >= 8, "second phase should be >= 8ms, got {phase2}");

        let total = timer.total_elapsed_ms();
        assert!(total >= 10, "total should be >= 10ms, got {total}");
    }

    #[test]
    fn metrics_status_bar_text() {
        let metrics = StartupMetrics {
            config_load_ms: 5,
            tui_init_ms: 20,
            db_open_ms: 15,
            total_ms: 42,
        };
        assert_eq!(metrics.status_bar_text(), "startup: 42ms");
    }
}
