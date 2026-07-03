//! Short activity labels used while the agent is producing a response.

use crate::app::AgentState;
use crate::app::CodingMode;

pub fn agent_activity_label(state: &AgentState, mode: CodingMode) -> &'static str {
    match state {
        AgentState::RunningTool(_) => "working",
        AgentState::Error(_) => "error",
        AgentState::Idle => "idle",
        AgentState::Thinking => mode_activity_label(mode),
    }
}

fn mode_activity_label(mode: CodingMode) -> &'static str {
    match mode {
        CodingMode::Plan => "planning",
        CodingMode::Vibe => "iterating",
    }
}

#[cfg(test)]
mod tests {
    use super::agent_activity_label;
    use crate::app::AgentState;
    use crate::app::CodingMode;

    #[test]
    fn thinking_label_follows_coding_mode() {
        assert_eq!(
            agent_activity_label(&AgentState::Thinking, CodingMode::Plan),
            "planning"
        );
        assert_eq!(
            agent_activity_label(&AgentState::Thinking, CodingMode::Vibe),
            "iterating"
        );
    }

    #[test]
    fn running_tool_uses_working_label() {
        assert_eq!(
            agent_activity_label(&AgentState::RunningTool("edit".into()), CodingMode::Plan),
            "working"
        );
    }
}
