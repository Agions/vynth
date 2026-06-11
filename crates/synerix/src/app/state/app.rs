//! App struct — global application state + constructors.

use crate::agent::CustomAgentRegistry;
use crate::coding_modes::CodingMode;
use crate::config::{KeyBindings, KeymapProfile, Settings};
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
    /// Stored layout rects from last draw pass
    pub layout_state: LayoutState,
    /// Agent event receiver
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
}

impl App {
    /// Create App with external channel (for testing)
    pub fn new_with_channel(
        settings: Settings,
        _agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
        config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
        mode: InputMode,
    ) -> Self {
        let keybindings = Self::create_keybindings(&settings);
        Self {
            mode,
            dirty_flags: DirtyFlags::all_dirty(),
            focused_panel: FocusedPanel::Input,
            chat_state: ChatState::default(),
            sidebar_state: SidebarState::default(),
            diff_state: DiffState::default(),
            status_bar: StatusBarState::default(),
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings,
            yank_buffer: String::new(),
            layout_state: LayoutState::default(),
            settings,
            should_quit: false,
            agent_rx,
            config_reload_rx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
            goal_state: GoalState::inactive(),
            coding_mode: CodingMode::Act,
        }
    }

    #[allow(dead_code)]
    pub fn new(settings: Settings) -> Self {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        Self::new_with_channel(
            settings,
            agent_tx,
            agent_rx,
            config_reload_rx,
            InputMode::Insert,
        )
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

        // Handle slash commands
        if crate::slash::try_handle(self, &text) {
            return;
        }

        self.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text,
            tool_calls: Vec::new(),
        });
        // Reset scroll to bottom on new message
        self.chat_state.scroll_offset = 0;
    }

    /// Get the byte position of the previous character boundary
    pub(crate) fn prev_char_pos(&self) -> usize {
        self.input_buffer[..self.input_cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Get the byte position of the next character boundary
    pub(crate) fn next_char_pos(&self) -> usize {
        self.input_buffer[self.input_cursor..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.input_cursor + i)
            .unwrap_or(self.input_buffer.len())
    }
}
