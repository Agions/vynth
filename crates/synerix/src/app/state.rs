//! Application state types — structs, enums, and their constructors.

use crate::agent::CustomAgentRegistry;
use crate::coding_modes::CodingMode;
use crate::config::{KeyBindings, KeymapProfile, Settings};
use crate::skills::SkillRegistry;
use ratatui::layout::Rect;

use super::events::AgentEvent;
use super::message::{ChatMessage, MessageRole};

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
    /// Agent event sender — kept for future extensibility
    #[allow(dead_code)]
    pub(crate) agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
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
    #[allow(dead_code)]
    pub path: String,
    pub is_dir: bool,
    pub depth: usize,
}

/// Diff preview state
#[allow(dead_code)]
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
    pub goal_active: bool,
    pub goal_duration: String,
    /// Active coding mode indicator
    pub coding_mode: CodingMode,
}

/// /goal state — completion condition + auto-loop tracking
#[derive(Debug, Clone)]
pub struct GoalState {
    /// The condition text (e.g. "all tests in test/auth pass")
    pub condition: Option<String>,
    /// Turns evaluated so far
    pub turns: u32,
    /// When the goal was set (Unix timestamp)
    pub started_at: Option<i64>,
    /// The evaluator's last reason
    pub last_reason: String,
    /// Whether the goal was achieved
    pub achieved: bool,
}

impl GoalState {
    pub fn inactive() -> Self {
        Self {
            condition: None,
            turns: 0,
            started_at: None,
            last_reason: String::new(),
            achieved: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.condition.is_some() && !self.achieved
    }

    /// Human-readable duration string
    pub fn duration_str(&self) -> String {
        match self.started_at {
            None => String::new(),
            Some(start) => {
                let elapsed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
                    - start;
                let mins = elapsed / 60;
                let secs = elapsed % 60;
                if mins > 0 {
                    format!("{}m{}s", mins, secs)
                } else {
                    format!("{}s", secs)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AgentState {
    Idle,
    Thinking,
    RunningTool(String),
    Error(String),
}

// ── Dirty Flags ──────────────────────────────────────────

/// Per-widget dirty flags for differential rendering
#[derive(Debug, Default, Clone, Copy)]
pub struct DirtyFlags {
    pub sidebar: bool,
    pub chat: bool,
    pub diff: bool,
    pub input: bool,
    pub status: bool,
}

impl DirtyFlags {
    pub fn all_dirty() -> Self {
        Self {
            sidebar: true,
            chat: true,
            diff: true,
            input: true,
            status: true,
        }
    }
    #[allow(dead_code)]
    pub fn is_clean(&self) -> bool {
        !self.sidebar && !self.chat && !self.diff && !self.input && !self.status
    }
}

// ── Constructors ──────────────────────────────────────────────

impl App {
    /// Create App with external channel (for testing)
    #[allow(dead_code)]
    pub fn new_with_channel(
        settings: Settings,
        agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    ) -> Self {
        let (_config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let keybindings = Self::create_keybindings(&settings);
        Self {
            mode: InputMode::Normal,
            dirty_flags: DirtyFlags::all_dirty(),
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
                goal_active: false,
                goal_duration: String::new(),
                coding_mode: CodingMode::Act,
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
            goal_state: GoalState::inactive(),
            coding_mode: CodingMode::Act,
        }
    }

    pub fn new(settings: Settings) -> Self {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
        let keybindings = Self::create_keybindings(&settings);
        let model_name = settings.llm.model.clone();

        Self {
            mode: InputMode::Insert,
            dirty_flags: DirtyFlags::all_dirty(),
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
                goal_active: false,
                goal_duration: String::new(),
                coding_mode: CodingMode::Act,
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
            goal_state: GoalState::inactive(),
            coding_mode: CodingMode::Act,
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
