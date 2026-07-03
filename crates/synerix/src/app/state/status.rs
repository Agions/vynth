//! Status bar and agent state types.

use std::time::Instant;

use crate::config::SandboxMode;

/// Status bar state
#[derive(Debug, Clone, Default)]
pub struct StatusBarState {
    pub agent_state: AgentState,
    pub model_name: String,
    pub tokens_used: usize,
    pub tokens_total: usize,
    pub sandbox_mode: SandboxMode,
    pub startup_metrics: Option<crate::telemetry::StartupMetrics>,
    pub goal_active: bool,
    pub goal_duration: String,
    /// Monotonic frame counter used for subtle TUI animation.
    pub animation_frame: u64,
    /// When the current agent activity started (for elapsed timer display).
    pub agent_start_time: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
pub enum AgentState {
    #[default]
    Idle,
    Thinking,
    RunningTool(String),
    Error(String),
}
