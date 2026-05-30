//! Application controller — global state machine + event dispatch

use crate::config::Settings;
use crate::error::AppError;
use crate::tui::event::AppEvent;

/// Run the application
pub async fn run(settings: Settings) -> Result<(), AppError> {
    tracing::info!("Initializing application");

    let mut app = App::new(settings);

    // Initialize TUI
    let mut terminal = crate::tui::init()?;

    let result = app.run(&mut terminal).await;

    // Restore terminal
    crate::tui::restore(terminal)?;

    result
}

/// Global application state
pub struct App {
    /// Current input mode
    pub mode: InputMode,
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
    /// Agent event receiver
    agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    /// Agent event sender (cloned for agent tasks)
    agent_tx: tokio::sync::mpsc::UnboundedSender<AgentEvent>,
}

/// Input modes (Vim-inspired)
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
        Self {
            mode: InputMode::Normal,
            chat_state: ChatState {
                messages: Vec::new(),
                streaming_text: String::new(),
                is_streaming: false,
                scroll_offset: 0,
            },
            sidebar_state: SidebarState {
                active_tab: SidebarTab::Files,
                file_tree: Vec::new(),
            },
            diff_state: DiffState {
                visible: false,
                content: String::new(),
                hunks: Vec::new(),
            },
            status_bar: StatusBarState {
                model_name: settings.llm.model.clone(),
                tokens_used: 0,
                tokens_total: settings.llm.context_window,
                agent_state: AgentState::Idle,
                sandbox_mode: format!("{:?}", settings.sandbox.mode),
            },
            input_buffer: String::new(),
            input_cursor: 0,
            settings,
            should_quit: false,
            agent_rx,
            agent_tx,
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

        let model_name = settings.llm.model.clone();

        Self {
            mode: InputMode::Insert,
            chat_state: ChatState {
                messages: Vec::new(),
                streaming_text: String::new(),
                is_streaming: false,
                scroll_offset: 0,
            },
            sidebar_state: SidebarState {
                active_tab: SidebarTab::Files,
                file_tree: Vec::new(),
            },
            diff_state: DiffState {
                visible: false,
                content: String::new(),
                hunks: Vec::new(),
            },
            status_bar: StatusBarState {
                agent_state: AgentState::Idle,
                model_name,
                tokens_used: 0,
                tokens_total: 0,
                sandbox_mode: "confirm".to_string(),
            },
            input_buffer: String::new(),
            input_cursor: 0,
            settings,
            should_quit: false,
            agent_rx,
            agent_tx,
        }
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
            terminal.draw(|frame| self.draw(frame))?;

            // Wait for: user input OR agent event
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
                // Global keybindings
                if key.code == crossterm::event::KeyCode::Char('c')
                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    self.should_quit = true;
                    return Ok(());
                }

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
            AppEvent::Mouse(_mouse) => {
                // Mouse events handled later
            }
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

    /// Draw the TUI frame
    fn draw(&self, frame: &mut ratatui::Frame) {
        crate::tui::frame::draw_frame(frame, self);
    }
}
