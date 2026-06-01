//! TUI event source — crossterm keyboard/mouse → AppEvent

use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::time::Duration;

/// Application events
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

/// Poll for the next event (non-blocking with tick interval)
pub async fn poll_event() -> Option<AppEvent> {
    // Poll with a short timeout to allow tick events
    if event::poll(Duration::from_millis(50)).ok()? {
        match event::read().ok()? {
            Event::Key(key) => Some(AppEvent::Key(key)),
            Event::Mouse(mouse) => Some(AppEvent::Mouse(mouse)),
            Event::Resize(w, h) => Some(AppEvent::Resize(w, h)),
            _ => None,
        }
    } else {
        Some(AppEvent::Tick)
    }
}
