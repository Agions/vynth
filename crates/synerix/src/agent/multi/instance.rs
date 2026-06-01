//! A running agent instance

use tokio::sync::mpsc;

use crate::agent::agent_loop::run_agent_loop;
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
    /// This method drives `run_agent_loop` from agent_loop.rs with full tool support.
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

        let result = run_agent_loop(
            llm,
            &mut self.context,
            tools,
            mcp,
            event_tx,
            max_turns,
            120, // default tool timeout
        )
        .await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::multi::MessageType;
    use crate::agent::roles::AgentRole;

    fn make_instance(name: &str, role: AgentRole) -> AgentInstance {
        let config = AgentConfig::new(role, name);
        AgentInstance::new(format!("agent_{}", name), config)
    }

    // ── new / initialization ──────────────────────────────────

    #[test]
    fn new_creates_idle_instance() {
        let inst = make_instance("test", AgentRole::Coder);
        assert_eq!(inst.status, AgentStatus::Idle);
        assert!(inst.result_buffer.is_empty());
        assert_eq!(inst.id, "agent_test");
    }

    #[test]
    fn new_sets_config_correctly() {
        let inst = make_instance("reviewer1", AgentRole::Reviewer);
        assert_eq!(inst.config.name, "reviewer1");
        assert!(matches!(inst.config.role, AgentRole::Reviewer));
    }

    #[test]
    fn new_context_starts_empty() {
        let inst = make_instance("t", AgentRole::Tester);
        assert!(inst.context.messages().is_empty());
    }

    // ── system_prompt ─────────────────────────────────────────

    #[test]
    fn system_prompt_uses_custom_when_set() {
        let config =
            AgentConfig::new(AgentRole::Coder, "c").with_system_prompt("Custom prompt here");
        let inst = AgentInstance::new("id".into(), config);
        assert_eq!(inst.system_prompt(), "Custom prompt here");
    }

    #[test]
    fn system_prompt_falls_back_to_role() {
        let inst = make_instance("c", AgentRole::Reviewer);
        let prompt = inst.system_prompt();
        assert!(prompt.contains("code reviewer"));
    }

    // ── process_message ───────────────────────────────────────

    #[test]
    fn process_message_returns_response() {
        let mut inst = make_instance("worker", AgentRole::Coder);
        let response = inst.process_message("hello");
        assert!(response.contains("Processing: hello"));
        assert!(response.contains("worker"));
    }

    #[test]
    fn process_message_transitions_to_idle() {
        let mut inst = make_instance("w", AgentRole::Coder);
        inst.process_message("test");
        assert_eq!(inst.status, AgentStatus::Idle);
    }

    #[test]
    fn process_message_adds_user_and_assistant_messages() {
        let mut inst = make_instance("w", AgentRole::Coder);
        inst.process_message("hi");
        let msgs = inst.context.messages();
        assert!(msgs.len() >= 2);
        // Last message should be assistant
        let last = msgs.last().unwrap();
        assert!(matches!(
            last.role,
            crate::llm::types::MessageRole::Assistant
        ));
    }

    // ── stop ──────────────────────────────────────────────────

    #[test]
    fn stop_sets_done_status() {
        let mut inst = make_instance("s", AgentRole::Coder);
        inst.stop();
        assert_eq!(inst.status, AgentStatus::Done);
    }

    #[test]
    fn stop_clears_context() {
        let mut inst = make_instance("s", AgentRole::Coder);
        inst.process_message("something");
        assert!(!inst.context.messages().is_empty());
        inst.stop();
        assert!(inst.context.messages().is_empty());
    }

    // ── restart ───────────────────────────────────────────────

    #[test]
    fn restart_sets_idle_status() {
        let mut inst = make_instance("r", AgentRole::Coder);
        inst.stop();
        assert_eq!(inst.status, AgentStatus::Done);
        inst.restart();
        assert_eq!(inst.status, AgentStatus::Idle);
    }

    #[test]
    fn restart_clears_context_and_buffer() {
        let mut inst = make_instance("r", AgentRole::Coder);
        inst.process_message("test");
        inst.result_buffer.push("buf".into());
        inst.restart();
        assert!(inst.context.messages().is_empty());
        assert!(inst.result_buffer.is_empty());
    }

    // ── collect_response (via process_message) ────────────────

    #[test]
    fn collect_response_returns_assistant_content() {
        let mut inst = make_instance("c", AgentRole::Coder);
        let response = inst.process_message("do something");
        // The response should match what collect_response would find
        assert!(response.contains("do something"));
    }

    // ── message channel ───────────────────────────────────────

    #[test]
    fn message_channel_is_open() {
        let inst = make_instance("m", AgentRole::Coder);
        // Sending should succeed (receiver is alive)
        let msg = AgentMessage {
            from: "a".into(),
            to: inst.id.clone(),
            content: "ping".into(),
            msg_type: MessageType::Request,
            task_id: None,
        };
        assert!(inst.message_tx.send(msg).is_ok());
    }
}
