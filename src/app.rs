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
            settings,
            should_quit: false,
            agent_rx,
            agent_tx,
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
                // Submit current input
                // TODO: collect input buffer and send to agent
                tracing::debug!("Enter pressed — submit input");
            }
            _ => {
                // Append to input buffer
                // TODO: implement input buffer
            }
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
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                // Execute command
                // TODO: parse and execute :commands
                self.mode = InputMode::Normal;
            }
            _ => {
                // Append to command buffer
            }
        }
        Ok(())
    }

    async fn handle_search_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<(), AppError> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.mode = InputMode::Normal;
            }
            crossterm::event::KeyCode::Enter => {
                // Execute search
                self.mode = InputMode::Normal;
            }
            _ => {
                // Append to search buffer
            }
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
