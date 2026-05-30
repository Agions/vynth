//! Application controller — global state machine + event dispatch

use crate::config::keymap::Action;
use crate::config::{KeyBindings, KeymapProfile, Settings};
use crate::error::AppError;
use crate::tui::event::AppEvent;
use ratatui::layout::Rect;

/// Run the application
pub async fn run(settings: Settings, startup_metrics: crate::telemetry::StartupMetrics) -> Result<(), AppError> {
    tracing::info!("Initializing application");

    let mut app = App::new(settings);

    // Attach startup metrics to status bar
    app.status_bar.startup_metrics = Some(startup_metrics);

    // Initialize TUI
    let mut terminal = crate::tui::init()?;

    let result = app.run(&mut terminal).await;

    // Restore terminal
    crate::tui::restore(terminal)?;

    result
}

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
    agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    /// Agent event sender (cloned for agent tasks)
    agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
    /// Config reload receiver (from watcher/SIGHUP)
    config_reload_rx: tokio::sync::mpsc::UnboundedReceiver<crate::config::ConfigReload>,
    /// Config version counter — incremented on each hot-reload
    pub config_version: u64,
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

impl App {
    /// Create App with external channel (for testing)
    pub fn new_with_channel(
        settings: crate::config::Settings,
        agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
        agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    ) -> Self {
        let (config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
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
        }
    }
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
    ToolCallStart { name: String, args: serde_json::Value },
    ToolResult { name: String, output: String, is_error: bool },
    Done,
    Error(String),
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (config_reload_tx, config_reload_rx) = tokio::sync::mpsc::unbounded_channel();
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
        }
    }

    fn create_keybindings(settings: &Settings) -> KeyBindings {
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
        if !text.is_empty() {
            self.chat_state.messages.push(ChatMessage {
                role: MessageRole::User,
                content: text,
                tool_calls: Vec::new(),
            });
            // Reset scroll to bottom on new message
            self.chat_state.scroll_offset = 0;
        }
    }

    /// Main event loop
    pub async fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), AppError> {
        tracing::info!("Entering main event loop");

        loop {
            // Render frame
            terminal.draw(|frame| self.draw_with_layout(frame))?;

            // Wait for: user input, agent event, or config reload
            tokio::select! {
                event = crate::tui::event::poll_event() => {
                    if let Some(event) = event {
                        self.handle_input(event).await?;
                    }
                }
                agent_event = self.agent_rx.recv() => {
                    if let Some(event) = agent_event {
                        self.handle_agent_event(event);
                    }
                }
                reload = self.config_reload_rx.recv() => {
                    if let Some(reload) = reload {
                        self.apply_config_reload(reload);
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle user input events
    async fn handle_input(&mut self, event: AppEvent) -> Result<(), AppError> {
        match event {
            AppEvent::Key(key) => {
                // Global keybindings (Ctrl+C always quits)
                if key.code == crossterm::event::KeyCode::Char('c')
                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    self.should_quit = true;
                    return Ok(());
                }

                // Try keybinding resolution first
                let action = self.keybindings.resolve(&self.mode, key);
                if action != Action::Noop {
                    return self.execute_action(action).await;
                }

                // Fallback to existing match arms
                match self.mode {
                    InputMode::Insert => self.handle_insert_key(key).await?,
                    InputMode::Normal => self.handle_normal_key(key).await?,
                    InputMode::Command => self.handle_command_key(key).await?,
                    InputMode::Search => self.handle_search_key(key).await?,
                }
            }
            AppEvent::Resize(_, _) => {
                // ratatui handles this automatically
            }
            AppEvent::Tick => {
                // Periodic update (e.g., cursor blink)
            }
            AppEvent::Mouse(mouse) => {
                self.handle_mouse(mouse);
            }
        }
        Ok(())
    }

    /// Handle mouse click/scroll events
    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        let x = mouse.column;
        let y = mouse.row;
        let layout = self.layout_state.clone();

        let in_rect = |r: &Rect| x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if in_rect(&layout.sidebar_rect) {
                    self.focused_panel = FocusedPanel::Sidebar;
                    // Sidebar tab bar is in the first 2 rows (1 row + border)
                    let tab_click_y = y.saturating_sub(layout.sidebar_rect.y);
                    if tab_click_y <= 1 {
                        // Tab header area — switch tab based on X position
                        let rel_x = x.saturating_sub(layout.sidebar_rect.x + 1) as usize;
                        if rel_x < 6 {
                            self.sidebar_state.active_tab = SidebarTab::Files;
                        } else if rel_x < 15 {
                            self.sidebar_state.active_tab = SidebarTab::Sessions;
                        } else {
                            self.sidebar_state.active_tab = SidebarTab::Skills;
                        }
                    } else {
                        // Content area click — select item by row
                        let content_row = tab_click_y.saturating_sub(2) as usize;
                        self.select_sidebar_item(content_row);
                    }
                } else if in_rect(&layout.input_rect) {
                    self.focused_panel = FocusedPanel::Input;
                    self.mode = InputMode::Insert;
                    // Position cursor at click X relative to input inner area
                    let inner_x = x.saturating_sub(layout.input_rect.x + 1) as usize;
                    let byte_pos = self
                        .input_buffer
                        .char_indices()
                        .nth(inner_x)
                        .map(|(i, _)| i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_cursor = byte_pos;
                } else if in_rect(&layout.chat_rect) {
                    self.focused_panel = FocusedPanel::Chat;
                    self.mode = InputMode::Insert;
                } else if in_rect(&layout.diff_rect) {
                    self.focused_panel = FocusedPanel::Diff;
                } else if in_rect(&layout.status_rect) {
                    // No action on status bar click
                }
            }
            MouseEventKind::ScrollUp => {
                if in_rect(&layout.chat_rect) {
                    let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                    self.chat_state.scroll_offset =
                        (self.chat_state.scroll_offset + 3).min(max_scroll);
                } else if in_rect(&layout.diff_rect) {
                    self.diff_state.scroll_offset += 3;
                } else if in_rect(&layout.sidebar_rect) {
                    self.sidebar_state.scroll_offset += 3;
                }
            }
            MouseEventKind::ScrollDown => {
                if in_rect(&layout.chat_rect) {
                    self.chat_state.scroll_offset =
                        self.chat_state.scroll_offset.saturating_sub(3);
                } else if in_rect(&layout.diff_rect) {
                    self.diff_state.scroll_offset =
                        self.diff_state.scroll_offset.saturating_sub(3);
                } else if in_rect(&layout.sidebar_rect) {
                    self.sidebar_state.scroll_offset =
                        self.sidebar_state.scroll_offset.saturating_sub(3);
                }
            }
            _ => {}
        }
    }

    /// Select a sidebar item by content row index
    fn select_sidebar_item(&mut self, row: usize) {
        let actual_row = row + self.sidebar_state.scroll_offset;
        match self.sidebar_state.active_tab {
            SidebarTab::Files => {
                if actual_row < self.sidebar_state.file_tree.len() {
                    tracing::debug!(
                        "Selected file: {}",
                        self.sidebar_state.file_tree[actual_row].name
                    );
                }
            }
            SidebarTab::Sessions => {
                tracing::debug!("Selected session row {}", actual_row);
            }
            SidebarTab::Skills => {
                tracing::debug!("Selected skill row {}", actual_row);
            }
        }
    }

    /// Execute a resolved Action
    async fn execute_action(&mut self, action: Action) -> Result<(), AppError> {
        match action {
            // Text editing
            Action::InsertChar(c) => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            Action::DeleteChar => {
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
            }
            Action::DeleteCharForward => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.input_buffer[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_buffer.replace_range(self.input_cursor..next, "");
                }
            }
            Action::DeleteWord => {
                // Delete word backwards
                if self.input_cursor > 0 {
                    let before = &self.input_buffer[..self.input_cursor];
                    let trimmed = before.trim_end();
                    let new_pos = trimmed.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                    self.input_buffer.replace_range(new_pos..self.input_cursor, "");
                    self.input_cursor = new_pos;
                }
            }
            Action::KillToEnd => {
                self.input_buffer.truncate(self.input_cursor);
            }
            Action::KillToStart => {
                self.yank_buffer = self.input_buffer[..self.input_cursor].to_string();
                self.input_buffer.replace_range(..self.input_cursor, "");
                self.input_cursor = 0;
            }
            Action::MoveCursorLeft => {
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_cursor = prev;
                }
            }
            Action::MoveCursorRight => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.input_buffer[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_cursor = next;
                }
            }
            Action::MoveCursorHome => {
                self.input_cursor = 0;
            }
            Action::MoveCursorEnd => {
                self.input_cursor = self.input_buffer.len();
            }

            // Mode transitions
            Action::SubmitMessage => {
                self.submit_message();
            }
            Action::EnterInsertMode => {
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeAppend => {
                // Move cursor right one char, then enter insert
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.input_buffer[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_cursor = next;
                }
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeOpenLineBelow => {
                // Move to end, add newline, enter insert
                self.input_cursor = self.input_buffer.len();
                self.input_buffer.push('\n');
                self.input_cursor = self.input_buffer.len();
                self.mode = InputMode::Insert;
            }
            Action::EnterInsertModeOpenLineAbove => {
                // Add newline at current position, enter insert
                self.input_buffer.insert(self.input_cursor, '\n');
                // Cursor stays at the inserted newline position
                self.mode = InputMode::Insert;
            }
            Action::EnterNormalMode => {
                self.mode = InputMode::Normal;
                // Move cursor back one if possible (vim convention)
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_cursor = prev;
                }
            }
            Action::EnterCommandMode => {
                self.mode = InputMode::Command;
            }
            Action::EnterSearchMode => {
                self.mode = InputMode::Search;
            }

            // Scrolling
            Action::ScrollUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
            }
            Action::ScrollDown => {
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
            }
            Action::ScrollToBottom => {
                self.chat_state.scroll_offset = 0;
            }
            Action::ScrollPageUp => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_add(10);
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset > max_scroll {
                    self.chat_state.scroll_offset = max_scroll;
                }
            }
            Action::ScrollPageDown => {
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(10);
            }

            // Application
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Cancel => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            Action::TabNext => {
                // Cycle sidebar tabs
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Sessions,
                    SidebarTab::Sessions => SidebarTab::Skills,
                    SidebarTab::Skills => SidebarTab::Files,
                };
            }
            Action::TabPrev => {
                self.sidebar_state.active_tab = match self.sidebar_state.active_tab {
                    SidebarTab::Files => SidebarTab::Skills,
                    SidebarTab::Sessions => SidebarTab::Files,
                    SidebarTab::Skills => SidebarTab::Sessions,
                };
            }

            // Yank/paste
            Action::YankLine => {
                self.yank_buffer = self.input_buffer.clone();
            }
            Action::Paste => {
                let paste = self.yank_buffer.clone();
                self.input_buffer.insert_str(self.input_cursor, &paste);
                self.input_cursor += paste.len();
            }

            // Vim clear line (dd)
            Action::ClearLine => {
                self.yank_buffer = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_cursor = 0;
            }

            Action::Noop => {}
        }
        Ok(())
    }

    async fn handle_insert_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                self.submit_message();
            }
            crossterm::event::KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    // Find the previous character boundary
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
            }
            crossterm::event::KeyCode::Delete => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.input_buffer[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_buffer.replace_range(self.input_cursor..next, "");
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_cursor = prev;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.input_cursor < self.input_buffer.len() {
                    let next = self.input_buffer[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_buffer.len());
                    self.input_cursor = next;
                }
            }
            crossterm::event::KeyCode::Home => {
                self.input_cursor = 0;
            }
            crossterm::event::KeyCode::End => {
                self.input_cursor = self.input_buffer.len();
            }
            crossterm::event::KeyCode::Char(c) => {
                // Ignore Ctrl+<char> combos (already handled globally)
                if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.input_buffer.insert(self.input_cursor, c);
                    self.input_cursor += c.len_utf8();
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_normal_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Char('i') => {
                self.mode = InputMode::Insert;
            }
            crossterm::event::KeyCode::Char(':') => {
                self.mode = InputMode::Command;
            }
            crossterm::event::KeyCode::Char('/') => {
                self.mode = InputMode::Search;
            }
            crossterm::event::KeyCode::Char('q') => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Char('j') => {
                // Scroll down (older messages)
                let max_scroll = self.chat_state.messages.len().saturating_sub(1);
                if self.chat_state.scroll_offset < max_scroll {
                    self.chat_state.scroll_offset += 1;
                }
            }
            crossterm::event::KeyCode::Char('k') => {
                // Scroll up (newer messages)
                self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(1);
            }
            crossterm::event::KeyCode::Char('G') => {
                // Jump to bottom
                self.chat_state.scroll_offset = 0;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                // Execute command
                // TODO: parse and execute :commands
                tracing::debug!("Command: {}", self.input_buffer);
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
            }
            crossterm::event::KeyCode::Char(c) => {
                if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.input_buffer.insert(self.input_cursor, c);
                    self.input_cursor += c.len_utf8();
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_search_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                // Execute search
                tracing::debug!("Search: {}", self.input_buffer);
                self.input_buffer.clear();
                self.input_cursor = 0;
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    let prev = self.input_buffer[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_buffer.replace_range(prev..self.input_cursor, "");
                    self.input_cursor = prev;
                }
            }
            crossterm::event::KeyCode::Char(c) => {
                if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.input_buffer.insert(self.input_cursor, c);
                    self.input_cursor += c.len_utf8();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle agent streaming events
    fn handle_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::TextDelta(text) => {
                self.chat_state.streaming_text.push_str(&text);
                self.chat_state.is_streaming = true;
            }
            AgentEvent::ToolCallStart { name, args } => {
                self.status_bar.agent_state = AgentState::RunningTool(name.clone());
                tracing::info!("Tool call: {} with args: {}", name, args);
            }
            AgentEvent::ToolResult { name, output, is_error } => {
                tracing::info!(
                    "Tool result: {} (error={}): {}",
                    name,
                    is_error,
                    output.chars().take(100).collect::<String>()
                );
            }
            AgentEvent::Done => {
                if !self.chat_state.streaming_text.is_empty() {
                    self.chat_state.messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: std::mem::take(&mut self.chat_state.streaming_text),
                        tool_calls: Vec::new(),
                    });
                }
                self.chat_state.is_streaming = false;
                self.chat_state.scroll_offset = 0;
                self.status_bar.agent_state = AgentState::Idle;
            }
            AgentEvent::Error(msg) => {
                self.status_bar.agent_state = AgentState::Error(msg.clone());
                tracing::error!("Agent error: {}", msg);
            }
        }
    }

    /// Apply a config hot-reload — update live fields only
    fn apply_config_reload(&mut self, reload: crate::config::ConfigReload) {
        tracing::info!(
            version = reload.version,
            theme = %reload.settings.ui.theme,
            keymap = %reload.settings.ui.keymap,
            sandbox_mode = ?reload.settings.sandbox.mode,
            "Applying config hot-reload"
        );

        // Update keybindings first (needs &settings before any moves)
        self.keybindings = Self::create_keybindings(&reload.settings);

        // Update theme
        self.settings.ui.theme = reload.settings.ui.theme.clone();
        // Update keymap profile
        self.settings.ui.keymap = reload.settings.ui.keymap.clone();
        // Update sandbox mode
        self.settings.sandbox.mode = reload.settings.sandbox.mode.clone();
        self.status_bar.sandbox_mode = format!("{:?}", self.settings.sandbox.mode);

        self.config_version = reload.version;
    }

    /// Draw the TUI frame
    fn draw(&self, frame: &mut ratatui::Frame) {
        crate::tui::frame::draw_frame(frame, self);
    }

    /// Draw frame and store layout rects for mouse hit-testing
    fn draw_with_layout(&mut self, frame: &mut ratatui::Frame) {
        crate::tui::frame::draw_frame_with_layout(frame, self);
    }
}
