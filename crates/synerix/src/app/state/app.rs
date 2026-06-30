//! App struct — global application state + constructors.

use crate::agent::CustomAgentRegistry;
use crate::coding_modes::CodingMode;
use crate::config::{KeyBindings, KeymapProfile, Settings};
use crate::project::ProjectContext;
use crate::sandbox::approval::ApprovalDecision;
use crate::session::SessionStore;
use crate::skills::SkillRegistry;

use super::super::events::AgentEvent;
use super::super::message::{ChatMessage, MessageRole};
use super::chat::ChatState;
use super::diff::DiffState;
use super::dirty::DirtyFlags;
use super::goal::GoalState;
use super::input::InputMode;
use super::layout::{FocusedPanel, LayoutState};
use super::sidebar::SidebarState;
use super::slash_menu_state::SlashMenuState;
use super::status::StatusBarState;

/// Global application state
pub struct App {
    /// Dirty flags for differential rendering
    pub dirty_flags: DirtyFlags,
    /// Current input mode
    pub mode: InputMode,
    /// Which panel currently has focus
    pub focused_panel: FocusedPanel,
    /// Chat conversation state
    pub chat_state: ChatState,
    /// Sidebar state
    pub sidebar_state: SidebarState,
    /// Diff preview state
    pub diff_state: DiffState,
    /// Status bar state
    pub status_bar: StatusBarState,
    /// Application settings
    pub settings: Settings,
    /// Should quit flag
    pub should_quit: bool,
    /// Text input buffer
    pub input_buffer: String,
    /// Cursor position within input_buffer (byte offset)
    pub input_cursor: usize,
    /// Keybinding profile
    pub keybindings: KeyBindings,
    /// Yank/paste buffer
    pub yank_buffer: String,
    pub slash_menu_state: SlashMenuState,
    /// Stored layout rects from last draw pass
    pub layout_state: LayoutState,
    /// Agent event receiver
    pub(crate) agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    pub(crate) agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    /// Config reload receiver (from watcher/SIGHUP)
    pub(crate) config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
    /// Config version counter — incremented on each hot-reload
    pub config_version: u64,
    /// Skills loaded from `.synerix/skills/` (project-local skills)
    pub skill_registry: SkillRegistry,
    /// Custom agents loaded from `.synerix/agents/` (project-local agents)
    pub agent_registry: CustomAgentRegistry,
    /// Active /goal state for auto-loop behavior
    pub goal_state: GoalState,
    /// Active coding mode (Plan/Act/Chat/Architect)
    pub coding_mode: CodingMode,
    /// Project context detected at startup (language, type, tools)
    pub project_context: Option<ProjectContext>,
    /// Session store for persisting conversations
    pub session_store: Option<SessionStore>,
    /// Pending approval preview text (None = no pending approval)
    pub pending_approval: Option<String>,
    /// Sender for approval decisions (from TUI -> tool dispatch)
    pub approval_decision_tx: Option<tokio::sync::mpsc::Sender<ApprovalDecision>>,
}

impl App {
    /// Create App with external channel (for testing)
    pub fn new_with_channel(
        settings: Settings,
        agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
        config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
        mode: InputMode,
    ) -> Self {
        let keybindings = Self::create_keybindings(&settings);
        let status_bar = StatusBarState {
            model_name: settings.llm.model.clone(),
            tokens_total: settings.llm.context_window,
            sandbox_mode: settings.sandbox.mode.clone(),
            ..StatusBarState::default()
        };
        Self {
            mode,
            dirty_flags: DirtyFlags::all_dirty(),
            focused_panel: FocusedPanel::Input,
            chat_state: ChatState::default(),
            sidebar_state: SidebarState::default(),
            diff_state: DiffState::default(),
            status_bar,
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings,
            yank_buffer: String::new(),
            slash_menu_state: SlashMenuState::default(),
            layout_state: LayoutState::default(),
            settings,
            should_quit: false,
            agent_tx,
            agent_rx,
            config_reload_rx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
            goal_state: GoalState::inactive(),
            coding_mode: CodingMode::Act,
            project_context: None,
            session_store: None,
            pending_approval: None,
            approval_decision_tx: None,
        }
    }

    /// Create App with settings and a config reload channel (used by runner)
    pub fn new_with_settings(
        settings: Settings,
        config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
    ) -> Self {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        Self::new_with_channel(
            settings,
            agent_tx,
            agent_rx,
            config_reload_rx,
            InputMode::Insert,
        )
    }

    pub(crate) fn create_keybindings(settings: &Settings) -> KeyBindings {
        let profile = match settings.ui.keymap.as_str() {
            "vim" => KeymapProfile::Vim,
            "emacs" => KeymapProfile::Emacs,
            _ => KeymapProfile::Default,
        };
        KeyBindings::new(profile)
    }

    /// Submit the current input buffer as a user message
    pub fn submit_message(&mut self) {
        let text = std::mem::take(&mut self.input_buffer);
        self.input_cursor = 0;
        if text.is_empty() {
            return;
        }

        let command_text = crate::slash::normalize_command_input(&text);
        let handled = match command_text {
            Some(command) => crate::slash::try_handle(self, &command),
            None => crate::slash::try_handle(self, &text),
        };
        if handled {
            return;
        }

        self.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
            tool_calls: Vec::new(),
        });
        // Reset scroll to bottom on new message
        self.chat_state.scroll_offset = 0;
        self.status_bar.agent_state = super::status::AgentState::Thinking;
        self.spawn_agent_response(text);
    }

    fn spawn_agent_response(&self, text: String) {
        let settings = self.settings.clone();
        let event_tx = self.agent_tx.clone();
        let coding_mode = self.coding_mode;
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            tracing::warn!("No Tokio runtime available; agent response task not started");
            return;
        };

        handle.spawn(async move {
            if settings.llm.api_key.trim().is_empty() {
                let _ = event_tx.send(AgentEvent::Error(
                    "API key is not configured. Set SYNERIX_API_KEY or run /config show."
                        .to_string(),
                ));
                return;
            }

            let llm = crate::llm::create_provider(&settings.llm);
            let mut ctx = crate::agent::context::ContextManager::new(
                crate::agent::context::TokenBudget::from_config(&settings.llm),
            );
            let system_prompt = crate::agent::prompt::default_system_prompt();
            ctx.push(crate::llm::types::ChatMessage::system(
                &(system_prompt + coding_mode.system_prompt_suffix()),
            ));
            ctx.push(crate::llm::types::ChatMessage::user(&text));

            let mut tools = crate::tools::ToolRegistry::new();
            crate::tools::builtin::register_builtins(&mut tools);
            let mcp = match crate::mcp::McpManager::connect_all(&[]).await {
                Ok(mcp) => mcp,
                Err(err) => {
                    let _ = event_tx.send(AgentEvent::Error(err.to_string()));
                    return;
                }
            };

            if let Err(err) = crate::agent::run_agent_loop(
                llm.as_ref(),
                &mut ctx,
                &tools,
                &mcp,
                event_tx.clone(),
                8,
                settings.sandbox.tool_timeout_secs,
                coding_mode,
                None,
            )
            .await
            {
                let _ = event_tx.send(AgentEvent::Error(err.to_string()));
            }
        });
    }
}
