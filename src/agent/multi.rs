//! Multi-agent orchestration

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::agent::context::{ContextManager, TokenBudget};
use crate::agent::roles::{AgentCapabilities, AgentRole};
use crate::error::AppError;
use crate::llm::types::ChatMessage;

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

/// A running agent instance
pub struct AgentInstance {
    pub id: AgentId,
    pub config: AgentConfig,
    pub context: ContextManager,
    pub status: AgentStatus,
    pub message_rx: mpsc::UnboundedReceiver<AgentMessage>,
    pub message_tx: mpsc::UnboundedSender<AgentMessage>,
    pub result_buffer: Vec<String>,
}

impl AgentInstance {
    pub fn new(id: AgentId, config: AgentConfig) -> Self {
        let budget = TokenBudget::new(128_000);
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            id,
            config,
            context: ContextManager::new(budget),
            status: AgentStatus::Idle,
            message_rx: rx,
            message_tx: tx,
            result_buffer: Vec::new(),
        }
    }

    /// Get the effective system prompt
    pub fn system_prompt(&self) -> String {
        self.config
            .system_prompt
            .clone()
            .unwrap_or_else(|| self.config.role.system_prompt())
    }

    /// Push a user message and get the agent's response text
    pub fn process_message(&mut self, input: &str) -> String {
        self.context.push(ChatMessage::user(input));
        self.status = AgentStatus::Running;
        // In real usage, this would call run_agent_loop
        // For now, return a placeholder
        let response = format!("[{}] Processing: {}", self.config.name, input);
        self.context.push(ChatMessage::assistant(&response));
        self.status = AgentStatus::Idle;
        response
    }
}

/// Message bus for inter-agent communication
pub struct AgentBus {
    channels: HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>,
}

impl AgentBus {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent_id: AgentId, tx: mpsc::UnboundedSender<AgentMessage>) {
        self.channels.insert(agent_id, tx);
    }

    pub fn send(&self, msg: AgentMessage) -> Result<(), AppError> {
        if let Some(tx) = self.channels.get(&msg.to) {
            tx.send(msg)
                .map_err(|e| AppError::ExecutionFailed(format!("Send failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn broadcast(&self, from: &str, content: &str) -> Result<(), AppError> {
        for (id, tx) in &self.channels {
            if id != from {
                let msg = AgentMessage {
                    from: from.to_string(),
                    to: id.clone(),
                    content: content.to_string(),
                    msg_type: MessageType::Broadcast,
                    task_id: None,
                };
                let _ = tx.send(msg);
            }
        }
        Ok(())
    }
}

/// Manages a swarm of agents
pub struct AgentSwarm {
    agents: HashMap<AgentId, AgentInstance>,
    bus: AgentBus,
    next_id: usize,
}

impl AgentSwarm {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            bus: AgentBus::new(),
            next_id: 0,
        }
    }

    /// Spawn a new agent
    pub fn spawn_agent(&mut self, config: AgentConfig) -> AgentId {
        let id = format!("agent_{}", self.next_id);
        self.next_id += 1;

        let instance = AgentInstance::new(id.clone(), config);
        let tx = instance.message_tx.clone();

        self.bus.register(id.clone(), tx);
        self.agents.insert(id.clone(), instance);
        id
    }

    /// Get agent status
    pub fn agent_status(&self, id: &str) -> Option<&AgentStatus> {
        self.agents.get(id).map(|a| &a.status)
    }

    /// List all agent IDs
    pub fn agent_ids(&self) -> Vec<&AgentId> {
        self.agents.keys().collect()
    }

    /// Get agent role
    pub fn agent_role(&self, id: &str) -> Option<&AgentRole> {
        self.agents.get(id).map(|a| &a.config.role)
    }

    /// Coordinate a task across multiple agents (Orchestrator pattern)
    pub async fn coordinate(
        &mut self,
        task: &str,
        agent_ids: &[AgentId],
    ) -> Result<String, AppError> {
        let mut results = Vec::new();

        // Phase 1: Assign task to each agent sequentially
        for id in agent_ids {
            if let Some(agent) = self.agents.get_mut(id) {
                let result = agent.process_message(task);
                results.push(format!("[{}]: {}", id, result));
            }
        }

        // Phase 2: Collect and synthesize results
        let synthesis = results.join("\n\n");
        Ok(synthesis)
    }

    /// Run a simple single-agent task
    pub async fn run_task(
        &mut self,
        agent_id: &str,
        task: &str,
    ) -> Result<String, AppError> {
        let agent = self.agents.get_mut(agent_id)
            .ok_or_else(|| AppError::ExecutionFailed(format!("Agent {} not found", agent_id)))?;
        Ok(agent.process_message(task))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config() {
        let config = AgentConfig::new(AgentRole::Coder, "test-coder");
        assert_eq!(config.name, "test-coder");
        assert_eq!(config.role, AgentRole::Coder);
        assert_eq!(config.max_turns, 15);
    }

    #[test]
    fn test_agent_instance() {
        let config = AgentConfig::new(AgentRole::Reviewer, "reviewer-1");
        let instance = AgentInstance::new("agent_0".into(), config);
        assert_eq!(instance.status, AgentStatus::Idle);
        assert!(instance.system_prompt().contains("reviewer"));
    }

    #[test]
    fn test_agent_bus() {
        let mut bus = AgentBus::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);
        assert!(bus.channels.contains_key("agent_0"));
    }

    #[tokio::test]
    async fn test_swarm_spawn() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        assert_eq!(id, "agent_0");
        assert_eq!(swarm.agent_status(&id), Some(&AgentStatus::Idle));
        assert_eq!(swarm.agent_role(&id), Some(&AgentRole::Coder));
    }

    #[tokio::test]
    async fn test_swarm_coordinate() {
        let mut swarm = AgentSwarm::new();
        let id1 = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        let id2 = swarm.spawn_agent(AgentConfig::new(AgentRole::Reviewer, "reviewer"));

        let result = swarm.coordinate("Review this code", &[id1, id2]).await.unwrap();
        assert!(result.contains("agent_0"));
        assert!(result.contains("agent_1"));
    }

    #[tokio::test]
    async fn test_swarm_run_task() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Tester, "tester"));

        let result = swarm.run_task(&id, "Write tests").await.unwrap();
        assert!(result.contains("tester"));
    }
}
