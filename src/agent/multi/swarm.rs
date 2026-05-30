//! Manages a swarm of agents

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::agent::roles::AgentRole;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;

use super::bus::AgentBus;
use super::instance::AgentInstance;
use super::types::{AgentConfig, AgentId, AgentStatus, AgentSwarmEvent};

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
        self.agents
            .insert(id.clone(), Arc::new(Mutex::new(instance)));

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
        self.agents
            .get(id)
            .and_then(|a| a.try_lock().ok().map(|g| g.status.clone()))
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
            let result = agent.process_message_async(task, llm, tools, mcp).await;
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
    pub async fn run_task(&mut self, agent_id: &str, task: &str) -> Result<String, AppError> {
        let arc = self
            .agents
            .get(agent_id)
            .ok_or_else(|| AppError::ExecutionFailed(format!("Agent {} not found", agent_id)))?;
        let mut agent = arc.lock().await;
        Ok(agent.process_message(task))
    }
}
