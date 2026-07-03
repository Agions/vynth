//! TUI event source — crossterm keyboard/mouse → AppEvent.

use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use tokio::sync::mpsc;

/// Application events
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize,
    Tick,
}

pub fn spawn_event_reader() -> mpsc::UnboundedReceiver<AppEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while let Ok(evt) = event::read() {
            let app_event = match evt {
                Event::Key(key) => Some(AppEvent::Key(key)),
                Event::Mouse(mouse) => Some(AppEvent::Mouse(mouse)),
                Event::Resize(_w, _h) => Some(AppEvent::Resize),
                _ => None,
            };
            if let Some(app_event) = app_event {
                if tx.send(app_event).is_err() {
                    break;
                }
            }
        }
    });
    rx
}
