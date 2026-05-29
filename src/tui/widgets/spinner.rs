//! Spinner animation widget

use std::time::Instant;

pub struct Spinner {
    frames: Vec<&'static str>,
    frame_index: usize,
    last_update: Instant,
    interval_ms: u64,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            frame_index: 0,
            last_update: Instant::now(),
            interval_ms: 80,
        }
    }

    pub fn tick(&mut self) -> &str {
        if self.last_update.elapsed().as_millis() >= self.interval_ms as u128 {
            self.frame_index = (self.frame_index + 1) % self.frames.len();
            self.last_update = Instant::now();
        }
        self.frames[self.frame_index]
    }
}
