//! Goal state for auto-loop behavior.

#[derive(Debug, Clone, Default)]
pub struct GoalState {
    /// The condition text (e.g. "all tests in test/auth pass")
    pub condition: Option<String>,
    /// Turns evaluated so far
    pub turns: u32,
    /// When the goal was set (Unix timestamp)
    pub started_at: Option<i64>,
    /// The evaluator's last reason
    pub last_reason: String,
    /// Whether the goal was achieved
    pub achieved: bool,
}

impl GoalState {
    pub fn inactive() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.condition.is_some() && !self.achieved
    }

    /// Human-readable duration string
    pub fn duration_str(&self) -> String {
        match self.started_at {
            None => String::new(),
            Some(start) => {
                let elapsed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
                    - start;
                let mins = elapsed / 60;
                let secs = elapsed % 60;
                if mins > 0 {
                    format!("{}m{}s", mins, secs)
                } else {
                    format!("{}s", secs)
                }
            }
        }
    }
}
