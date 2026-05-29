//! Status bar widget

pub struct StatusBar {
    pub agent_state: AgentState,
    pub model_name: String,
    pub tokens_used: usize,
    pub tokens_total: usize,
    pub sandbox_mode: String,
}

pub enum AgentState {
    Idle,
    Thinking,
    RunningTool(String),
    Error(String),
}
