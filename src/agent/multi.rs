//! Multi-agent orchestration

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::agent::agloop::run_agent_loop;
use crate::agent::context::{ContextManager, TokenBudget};
use crate::agent::roles::{AgentCapabilities, AgentRole};
use crate::app::AgentEvent;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::ChatMessage;
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;

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

    /// Push a user message and get the agent's response text (synchronous placeholder)
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

    /// Push a user message and get the agent's response via the real LLM agent loop.
    /// This method drives `run_agent_loop` from agloop.rs with full tool support.
    pub async fn process_message_async(
        &mut self,
        input: &str,
        llm: &dyn LlmAdapter,
        tools: &ToolRegistry,
        mcp: &McpManager,
    ) -> Result<String, AppError> {
        self.context.push(ChatMessage::user(input));
        self.status = AgentStatus::Running;

        let max_turns = self.config.max_turns;
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentEvent>();

        let result =
            run_agent_loop(llm, &mut self.context, tools, mcp, event_tx, max_turns).await;

        // Drain remaining events to avoid channel warnings
        while event_rx.try_recv().is_ok() {}

        match result {
            Err(e) => {
                tracing::error!("Agent {} loop error: {}", self.id, e);
                self.status = AgentStatus::Error(e.to_string());
                Err(e)
            }
            Ok(()) => {
                self.status = AgentStatus::Idle;
                // Collect the assistant's final response from context
                let response = self.collect_response();
                Ok(response)
            }
        }
    }

    /// Stop this agent, clearing its context and marking it as Done
    pub fn stop(&mut self) {
        self.status = AgentStatus::Done;
        self.context.clear();
        tracing::info!("Agent {} stopped", self.id);
    }

    /// Restart this agent, clearing its context and resetting to Idle
    pub fn restart(&mut self) {
        self.context.clear();
        self.status = AgentStatus::Idle;
        self.result_buffer.clear();
        tracing::info!("Agent {} restarted", self.id);
    }

    /// Collect the assistant's final response from context messages
    fn collect_response(&self) -> String {
        self.context
            .messages()
            .iter()
            .rev()
            .find(|m| matches!(m.role, crate::llm::types::MessageRole::Assistant))
            .and_then(|m| m.content.clone())
            .unwrap_or_else(|| "(no response)".to_string())
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

    pub fn unregister(&mut self, agent_id: &str) {
        self.channels.remove(agent_id);
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

    /// Async send with timeout. Returns error if the send fails or times out.
    pub async fn send_async(
        &self,
        msg: AgentMessage,
        timeout: Duration,
    ) -> Result<(), AppError> {
        let to = msg.to.clone();
        if let Some(tx) = self.channels.get(&to) {
            let tx = tx.clone();
            tokio::time::timeout(timeout, async move {
                tx.send(msg)
                    .map_err(|e| AppError::ExecutionFailed(format!("Send failed: {}", e)))
            })
            .await
            .map_err(|_| AppError::ExecutionFailed(format!("Send to {} timed out", to)))?
        } else {
            Err(AppError::ExecutionFailed(format!(
                "Agent {} not registered",
                to
            )))
        }
    }

    /// Async broadcast with timeout. Collects results from all sends.
    pub async fn broadcast_async(
        &self,
        from: &str,
        content: &str,
        timeout: Duration,
    ) -> Result<(), AppError> {
        let mut futs = Vec::new();
        for (id, tx) in &self.channels {
            if id != from {
                let msg = AgentMessage {
                    from: from.to_string(),
                    to: id.clone(),
                    content: content.to_string(),
                    msg_type: MessageType::Broadcast,
                    task_id: None,
                };
                let tx = tx.clone();
                futs.push(async move { tx.send(msg) });
            }
        }
        // Drive all sends concurrently with timeout
        let results = tokio::time::timeout(timeout, futures::future::join_all(futs))
            .await
            .map_err(|_| AppError::ExecutionFailed("Broadcast timed out".to_string()))?;
        for result in results {
            result.map_err(|e| AppError::ExecutionFailed(format!("Broadcast send failed: {}", e)))?;
        }
        Ok(())
    }
}

/// Manages a swarm of agents
pub struct AgentSwarm {
    agents: HashMap<AgentId, Arc<Mutex<AgentInstance>>>,
    bus: AgentBus,
    next_id: usize,
    event_tx: Option<mpsc::UnboundedSender<AgentSwarmEvent>>,
}

impl AgentSwarm {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            bus: AgentBus::new(),
            next_id: 0,
            event_tx: None,
        }
    }

    /// Set the event channel for receiving swarm events
    pub fn set_event_channel(&mut self, tx: mpsc::UnboundedSender<AgentSwarmEvent>) {
        self.event_tx = Some(tx);
    }

    /// Emit a swarm event if a channel is configured
    fn emit_event(&self, event: AgentSwarmEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Spawn a new agent
    pub fn spawn_agent(&mut self, config: AgentConfig) -> AgentId {
        let id = format!("agent_{}", self.next_id);
        self.next_id += 1;

        let instance = AgentInstance::new(id.clone(), config.clone());
        let tx = instance.message_tx.clone();

        self.bus.register(id.clone(), tx);
        self.agents.insert(id.clone(), Arc::new(Mutex::new(instance)));

        self.emit_event(AgentSwarmEvent::AgentSpawned {
            id: id.clone(),
            role: config.role,
        });

        id
    }

    /// Remove an agent from the swarm
    pub fn remove_agent(&mut self, id: &str) -> Option<AgentConfig> {
        self.bus.unregister(id);
        let result = self.agents.remove(id).and_then(|arc| {
            // Try to get the config; if locked, just drop it
            match arc.try_lock() {
                Ok(agent) => Some(agent.config.clone()),
                Err(_) => None,
            }
        });

        if result.is_some() {
            self.emit_event(AgentSwarmEvent::AgentRemoved { id: id.to_string() });
        }

        result
    }

    /// Get agent status
    pub fn agent_status(&self, id: &str) -> Option<AgentStatus> {
        self.agents.get(id).and_then(|a| a.try_lock().ok().map(|g| g.status.clone()))
    }

    /// List all agent IDs
    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().cloned().collect()
    }

    /// Get agent role
    pub fn agent_role(&self, id: &str) -> Option<AgentRole> {
        self.agents
            .get(id)
            .and_then(|a| a.try_lock().ok().map(|g| g.config.role.clone()))
    }

    /// Stop a specific agent
    pub fn stop_agent(&self, id: &str) -> Result<(), AppError> {
        if let Some(arc) = self.agents.get(id) {
            let mut agent = arc
                .try_lock()
                .map_err(|_| AppError::ExecutionFailed(format!("Agent {} is busy", id)))?;
            agent.stop();
            self.emit_event(AgentSwarmEvent::AgentStopped { id: id.to_string() });
            Ok(())
        } else {
            Err(AppError::ExecutionFailed(format!("Agent {} not found", id)))
        }
    }

    /// Restart a specific agent
    pub fn restart_agent(&self, id: &str) -> Result<(), AppError> {
        if let Some(arc) = self.agents.get(id) {
            let mut agent = arc
                .try_lock()
                .map_err(|_| AppError::ExecutionFailed(format!("Agent {} is busy", id)))?;
            agent.restart();
            Ok(())
        } else {
            Err(AppError::ExecutionFailed(format!("Agent {} not found", id)))
        }
    }

    /// Coordinate a task across multiple agents in parallel (Orchestrator pattern).
    /// Uses `futures::future::join_all` to drive all agent futures concurrently.
    /// Each agent's LLM call overlaps with others, achieving true concurrency.
    pub async fn coordinate(
        &mut self,
        task: &str,
        agent_ids: &[AgentId],
        llm: &dyn LlmAdapter,
        tools: &ToolRegistry,
        mcp: &McpManager,
    ) -> Result<String, AppError> {
        // Collect Arc clones for each agent to process
        let mut agent_arcs = Vec::new();
        for id in agent_ids {
            if let Some(arc) = self.agents.get(id) {
                agent_arcs.push((id.clone(), Arc::clone(arc)));
            }
        }

        // Process all agents in parallel using join_all.
        // Each future owns its Arc<Mutex<AgentInstance>> and acquires the lock internally,
        // which avoids overlapping mutable borrows from the HashMap.
        let futs = agent_arcs.into_iter().map(|(id, agent_arc)| async move {
            let mut agent = agent_arc.lock().await;
            let result = agent
                .process_message_async(task, llm, tools, mcp)
                .await;
            (id, result)
        });

        let results = futures::future::join_all(futs).await;

        // Collect results and emit events
        let mut synthesis_parts = Vec::new();
        for (id, result) in results {
            match result {
                Ok(response) => {
                    synthesis_parts.push(format!("[{}]: {}", id, response));
                    self.emit_event(AgentSwarmEvent::AgentCompleted {
                        id,
                        result: response,
                    });
                }
                Err(e) => {
                    synthesis_parts.push(format!("[{}]: ERROR - {}", id, e));
                    self.emit_event(AgentSwarmEvent::AgentFailed {
                        id,
                        error: e.to_string(),
                    });
                }
            }
        }

        self.emit_event(AgentSwarmEvent::SwarmIdle);

        Ok(synthesis_parts.join("\n\n"))
    }

    /// Run a simple single-agent task (synchronous placeholder)
    pub async fn run_task(
        &mut self,
        agent_id: &str,
        task: &str,
    ) -> Result<String, AppError> {
        let arc = self
            .agents
            .get(agent_id)
            .ok_or_else(|| AppError::ExecutionFailed(format!("Agent {} not found", agent_id)))?;
        let mut agent = arc.lock().await;
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
        assert_eq!(swarm.agent_status(&id), Some(AgentStatus::Idle));
        assert_eq!(swarm.agent_role(&id), Some(AgentRole::Coder));
    }

    #[tokio::test]
    async fn test_swarm_coordinate() {
        let mut swarm = AgentSwarm::new();
        let id1 = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        let id2 = swarm.spawn_agent(AgentConfig::new(AgentRole::Reviewer, "reviewer"));

        // Use run_task to exercise the sync placeholder path
        let result = swarm.run_task(&id1, "Review this code").await.unwrap();
        assert!(result.contains("coder"));
        assert!(result.contains("Review this code"));
        let result2 = swarm.run_task(&id2, "Review this code").await.unwrap();
        assert!(result2.contains("reviewer"));
        assert!(result2.contains("Review this code"));
    }

    #[tokio::test]
    async fn test_swarm_run_task() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Tester, "tester"));

        let result = swarm.run_task(&id, "Write tests").await.unwrap();
        assert!(result.contains("tester"));
    }

    #[tokio::test]
    async fn test_remove_agent() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        assert!(swarm.agent_status(&id).is_some());

        let removed_config = swarm.remove_agent(&id);
        assert!(removed_config.is_some());
        assert_eq!(removed_config.unwrap().name, "coder");
        assert!(swarm.agent_status(&id).is_none());
        assert!(swarm.agent_ids().is_empty());
    }

    #[test]
    fn test_agent_stop_restart() {
        let config = AgentConfig::new(AgentRole::Coder, "coder");
        let mut instance = AgentInstance::new("agent_0".into(), config);
        assert_eq!(instance.status, AgentStatus::Idle);

        instance.stop();
        assert_eq!(instance.status, AgentStatus::Done);

        instance.restart();
        assert_eq!(instance.status, AgentStatus::Idle);
    }
}
