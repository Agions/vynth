//! Multi-agent type definitions

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
