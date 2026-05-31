//! Application state types — structs, enums, and their constructors.

use crate::agent::CustomAgentRegistry;
use crate::config::{KeyBindings, KeymapProfile, Settings};
use crate::skills::SkillRegistry;
use ratatui::layout::Rect;

/// Which panel currently has focus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPanel {
    Chat,
    Diff,
    Sidebar,
    Input,
}

/// Layout rects stored from the last render pass for mouse hit-testing
#[derive(Debug, Clone)]
pub struct LayoutState {
    pub sidebar_rect: Rect,
    pub chat_rect: Rect,
    pub diff_rect: Rect,
    pub input_rect: Rect,
    pub status_rect: Rect,
}

impl Default for LayoutState {
    fn default() -> Self {
        let zero = Rect::new(0, 0, 0, 0);
        Self {
            sidebar_rect: zero,
            chat_rect: zero,
            diff_rect: zero,
            input_rect: zero,
            status_rect: zero,
        }
    }
}

/// Global application state
pub struct App {
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
    /// Agent event sender (cloned for agent tasks)
    pub(crate) agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    /// Config reload receiver (from watcher/SIGHUP)
    pub(crate) config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
    /// Config version counter — incremented on each hot-reload
    pub config_version: u64,
    /// Skills loaded from `.synerix/skills/` (project-local skills)
    pub skill_registry: SkillRegistry,
    /// Custom agents loaded from `.synerix/agents/` (project-local agents)
    pub agent_registry: CustomAgentRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
    Search,
}

/// Chat conversation state
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub streaming_text: String,
    pub is_streaming: bool,
    /// Number of lines scrolled up from the bottom (0 = latest at bottom)
    pub scroll_offset: usize,
}

/// A single chat message
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Vec<ToolCallDisplay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool call display info
pub struct ToolCallDisplay {
    pub name: String,
    pub args_preview: String,
    pub result: Option<String>,
    pub is_error: bool,
}

/// Sidebar panel state
pub struct SidebarState {
    pub active_tab: SidebarTab,
    pub file_tree: Vec<FileEntry>,
    /// Scroll offset for file list
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarTab {
    Files,
    Sessions,
    Skills,
}

pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: usize,
}

/// Diff preview state
pub struct DiffState {
    pub visible: bool,
    pub content: String,
    pub hunks: Vec<DiffHunk>,
    /// Scroll offset for diff content
    pub scroll_offset: usize,
}

pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineKind {
    Add,
    Remove,
    Context,
}

/// Status bar state
pub struct StatusBarState {
    pub agent_state: AgentState,
    pub model_name: String,
    pub tokens_used: usize,
    pub tokens_total: usize,
    pub sandbox_mode: String,
    pub startup_metrics: Option<crate::telemetry::StartupMetrics>,
}

#[derive(Debug, Clone)]
pub enum AgentState {
    Idle,
    Thinking,
    RunningTool(String),
    Error(String),
}

/// Agent events (sent from agent task to TUI)
pub enum AgentEvent {
    TextDelta(String),
    ToolCallStart {
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        name: String,
        output: String,
        is_error: bool,
    },
    Done,
    Error(String),
}

// ── Constructors ──────────────────────────────────────────────

impl App {
    /// Create App with external channel (for testing)
    pub fn new_with_channel(
        settings: Settings,
        agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    ) -> Self {
        let (_config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let keybindings = Self::create_keybindings(&settings);
        Self {
            mode: InputMode::Normal,
            focused_panel: FocusedPanel::Input,
            chat_state: ChatState {
                messages: Vec::new(),
                streaming_text: String::new(),
                is_streaming: false,
                scroll_offset: 0,
            },
            sidebar_state: SidebarState {
                active_tab: SidebarTab::Files,
                file_tree: Vec::new(),
                scroll_offset: 0,
            },
            diff_state: DiffState {
                visible: false,
                content: String::new(),
                hunks: Vec::new(),
                scroll_offset: 0,
            },
            status_bar: StatusBarState {
                model_name: settings.llm.model.clone(),
                tokens_used: 0,
                tokens_total: settings.llm.context_window,
                agent_state: AgentState::Idle,
                sandbox_mode: format!("{:?}", settings.sandbox.mode),
                startup_metrics: None,
            },
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings,
            yank_buffer: String::new(),
            layout_state: LayoutState::default(),
            settings,
            should_quit: false,
            agent_rx,
            agent_tx,
            config_reload_rx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
        }
    }

    pub fn new(settings: Settings) -> Self {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let keybindings = Self::create_keybindings(&settings);
        let model_name = settings.llm.model.clone();

        Self {
            mode: InputMode::Insert,
            focused_panel: FocusedPanel::Input,
            chat_state: ChatState {
                messages: Vec::new(),
                streaming_text: String::new(),
                is_streaming: false,
                scroll_offset: 0,
            },
            sidebar_state: SidebarState {
                active_tab: SidebarTab::Files,
                file_tree: Vec::new(),
                scroll_offset: 0,
            },
            diff_state: DiffState {
                visible: false,
                content: String::new(),
                hunks: Vec::new(),
                scroll_offset: 0,
            },
            status_bar: StatusBarState {
                agent_state: AgentState::Idle,
                model_name,
                tokens_used: 0,
                tokens_total: 0,
                sandbox_mode: "confirm".to_string(),
                startup_metrics: None,
            },
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings,
            yank_buffer: String::new(),
            layout_state: LayoutState::default(),
            settings,
            should_quit: false,
            agent_rx,
            agent_tx,
            config_reload_rx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
        }
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
