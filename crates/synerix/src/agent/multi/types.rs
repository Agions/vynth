//! Multi-agent type definitions
// TODO: Multi-agent types — not yet wired
#![allow(dead_code)]

use crate::agent::roles::AgentRole;

/// Unique agent identifier
pub type AgentId = String;

/// Agent status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Done,
    Error(String),
}

/// Message type between agents
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// Direct request to an agent
    Request,
    /// Response from an agent
    Response,
    /// Broadcast to all agents
    Broadcast,
    /// Task delegation
    Delegate,
}

/// Inter-agent message
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub from: AgentId,
    pub to: AgentId,
    pub content: String,
    pub msg_type: MessageType,
    pub task_id: Option<String>,
}

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub role: AgentRole,
    pub name: String,
    pub system_prompt: Option<String>,
    pub tools_filter: Option<Vec<String>>,
    pub max_turns: usize,
}

impl AgentConfig {
    pub fn new(role: AgentRole, name: impl Into<String>) -> Self {
        let caps = role.default_capabilities();
        Self {
            role,
            name: name.into(),
            system_prompt: None,
            tools_filter: None,
            max_turns: caps.max_turns,
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }
}

/// Events emitted by the multi-agent swarm
#[derive(Debug, Clone)]
pub enum AgentSwarmEvent {
    AgentSpawned { id: AgentId, role: AgentRole },
    AgentCompleted { id: AgentId, result: String },
    AgentFailed { id: AgentId, error: String },
    AgentRemoved { id: AgentId },
    AgentStopped { id: AgentId },
    SwarmIdle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_variants() {
        assert_eq!(AgentStatus::Idle, AgentStatus::Idle);
        assert_eq!(AgentStatus::Running, AgentStatus::Running);
        assert_eq!(AgentStatus::Done, AgentStatus::Done);
        assert_ne!(AgentStatus::Idle, AgentStatus::Running);
    }

    #[test]
    fn test_agent_status_error() {
        let err = AgentStatus::Error("timeout".into());
        assert!(matches!(err, AgentStatus::Error(ref msg) if msg == "timeout"));
    }

    #[test]
    fn test_message_type_variants() {
        assert_eq!(MessageType::Request, MessageType::Request);
        assert_eq!(MessageType::Response, MessageType::Response);
        assert_eq!(MessageType::Broadcast, MessageType::Broadcast);
        assert_eq!(MessageType::Delegate, MessageType::Delegate);
        assert_ne!(MessageType::Request, MessageType::Response);
    }

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new(AgentRole::Coder, "test-coder");
        assert_eq!(config.name, "test-coder");
        assert!(matches!(config.role, AgentRole::Coder));
        assert!(config.system_prompt.is_none());
        assert!(config.tools_filter.is_none());
        assert!(config.max_turns > 0);
    }

    #[test]
    fn test_agent_config_with_system_prompt() {
        let config = AgentConfig::new(AgentRole::Reviewer, "reviewer")
            .with_system_prompt("You are a code reviewer");
        assert_eq!(config.system_prompt, Some("You are a code reviewer".into()));
    }

    #[test]
    fn test_agent_config_with_max_turns() {
        let config = AgentConfig::new(AgentRole::Tester, "tester").with_max_turns(5);
        assert_eq!(config.max_turns, 5);
    }

    #[test]
    fn test_agent_config_builder_chain() {
        let config = AgentConfig::new(AgentRole::Architect, "arch")
            .with_system_prompt("Design system")
            .with_max_turns(20);
        assert_eq!(config.system_prompt, Some("Design system".into()));
        assert_eq!(config.max_turns, 20);
    }

    #[test]
    fn test_agent_config_clone() {
        let config = AgentConfig::new(AgentRole::Coder, "c1");
        let cloned = config.clone();
        assert_eq!(cloned.name, config.name);
    }

    #[test]
    fn test_agent_message_clone() {
        let msg = AgentMessage {
            from: "a1".into(),
            to: "a2".into(),
            content: "hello".into(),
            msg_type: MessageType::Request,
            task_id: Some("task-1".into()),
        };
        let cloned = msg.clone();
        assert_eq!(cloned.from, "a1");
        assert_eq!(cloned.to, "a2");
        assert_eq!(cloned.content, "hello");
        assert_eq!(cloned.task_id, Some("task-1".into()));
    }

    #[test]
    fn test_agent_swarm_event_variants() {
        let spawned = AgentSwarmEvent::AgentSpawned {
            id: "a1".into(),
            role: AgentRole::Coder,
        };
        assert!(matches!(spawned, AgentSwarmEvent::AgentSpawned { .. }));

        let completed = AgentSwarmEvent::AgentCompleted {
            id: "a1".into(),
            result: "done".into(),
        };
        assert!(matches!(completed, AgentSwarmEvent::AgentCompleted { .. }));

        let failed = AgentSwarmEvent::AgentFailed {
            id: "a1".into(),
            error: "timeout".into(),
        };
        assert!(matches!(failed, AgentSwarmEvent::AgentFailed { .. }));

        let removed = AgentSwarmEvent::AgentRemoved { id: "a1".into() };
        assert!(matches!(removed, AgentSwarmEvent::AgentRemoved { .. }));

        let stopped = AgentSwarmEvent::AgentStopped { id: "a1".into() };
        assert!(matches!(stopped, AgentSwarmEvent::AgentStopped { .. }));

        let idle = AgentSwarmEvent::SwarmIdle;
        assert!(matches!(idle, AgentSwarmEvent::SwarmIdle));
    }
}
