//! A running agent instance

use tokio::sync::mpsc;

use crate::agent::agloop::run_agent_loop;
use crate::agent::context::{ContextManager, TokenBudget};
use crate::app::AgentEvent;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::ChatMessage;
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;

use super::types::{AgentConfig, AgentId, AgentMessage, AgentStatus};

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

        let result = run_agent_loop(llm, &mut self.context, tools, mcp, event_tx, max_turns).await;

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
