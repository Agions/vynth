//! TUI event source — crossterm keyboard/mouse → AppEvent
//!
//! Uses `tokio::task::spawn_blocking` for non-blocking terminal I/O.
//! This avoids the 50ms busy-poll pattern and lets the OS wake us on input.

use crossterm::event::{self, Event, KeyEvent, MouseEvent};

/// Application events
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

/// Read the next terminal event without busy-polling.
///
/// Uses `spawn_blocking` so the async runtime is not blocked by crossterm's
/// synchronous `event::read()`. The OS suspends the thread until input arrives,
/// consuming zero CPU while waiting.
pub async fn poll_event() -> Option<AppEvent> {
    tokio::task::spawn_blocking(|| event::read().ok())
        .await
        .ok()
        .flatten()
        .and_then(|evt| match evt {
            Event::Key(key) => Some(AppEvent::Key(key)),
            Event::Mouse(mouse) => Some(AppEvent::Mouse(mouse)),
            Event::Resize(w, h) => Some(AppEvent::Resize(w, h)),
            _ => None,
        })
}
