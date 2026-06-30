//! Status bar and agent state types.

use crate::coding_modes::CodingMode;
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
    /// Active coding mode indicator
    pub coding_mode: CodingMode,
    /// Monotonic frame counter used for subtle TUI animation.
    pub animation_frame: u64,
}

#[derive(Debug, Clone, Default)]
pub enum AgentState {
    #[default]
    Idle,
    Thinking,
    RunningTool(String),
    Error(String),
}
