//! Event loop + agent/config event handling.

use super::events::AgentEvent;
use super::message::{ChatMessage, MessageRole};
use super::state::{AgentState, App, FocusedPanel, InputMode, LayoutState, SidebarTab};
use crate::config::keymap::Action;
use crate::config::ConfigReload;
use crate::error::AppError;
use crate::tui::event::AppEvent;
use ratatui::layout::Rect;

impl App {
    /// Main event loop
    pub async fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), AppError> {
        tracing::info!("Entering main event loop");

        loop {
            terminal.draw(|frame| self.draw_with_layout(frame))?;

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
                // Ctrl+C always quits
                if key.code == crossterm::event::KeyCode::Char('c')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    self.should_quit = true;
                    return Ok(());
                }

                // Try keybinding resolution first
                let action = self.keybindings.resolve(&self.mode, key);
                if action != Action::Noop {
                    return self.execute_action(action).await;
                }

                // Delegate to mode-specific handler
                self.handle_mode_key(key).await?;
            }
            AppEvent::Resize(_, _) => {}
            AppEvent::Tick => {}
            AppEvent::Mouse(mouse) => {
                self.handle_mouse(mouse);
            }
        }
        Ok(())
    }

    // ── Mouse handling ──────────────────────────────────────

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        let x = mouse.column;
        let y = mouse.row;
        let layout = self.layout_state.clone();
        let in_rect = |r: &Rect| x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(x, y, &layout, &in_rect);
            }
            MouseEventKind::ScrollUp => {
                self.handle_mouse_scroll_up(&layout, &in_rect);
            }
            MouseEventKind::ScrollDown => {
                self.handle_mouse_scroll_down(&layout, &in_rect);
            }
            _ => {}
        }
    }

    fn handle_mouse_click(
        &mut self,
        x: u16,
        y: u16,
        layout: &LayoutState,
        in_rect: &dyn Fn(&Rect) -> bool,
    ) {
        if in_rect(&layout.sidebar_rect) {
            self.focused_panel = FocusedPanel::Sidebar;
            let tab_click_y = y.saturating_sub(layout.sidebar_rect.y);
            if tab_click_y <= 1 {
                let rel_x = x.saturating_sub(layout.sidebar_rect.x + 1) as usize;
                if rel_x < 6 {
                    self.sidebar_state.active_tab = SidebarTab::Files;
                } else if rel_x < 15 {
                    self.sidebar_state.active_tab = SidebarTab::Sessions;
                } else {
                    self.sidebar_state.active_tab = SidebarTab::Skills;
                }
            } else {
                let content_row = tab_click_y.saturating_sub(2) as usize;
                self.select_sidebar_item(content_row);
            }
        } else if in_rect(&layout.input_rect) {
            self.focused_panel = FocusedPanel::Input;
            self.mode = InputMode::Insert;
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
        }
    }

    fn handle_mouse_scroll_up(&mut self, layout: &LayoutState, in_rect: &dyn Fn(&Rect) -> bool) {
        if in_rect(&layout.chat_rect) {
            let max_scroll = self.chat_state.messages.len().saturating_sub(1);
            self.chat_state.scroll_offset = (self.chat_state.scroll_offset + 3).min(max_scroll);
        } else if in_rect(&layout.diff_rect) {
            self.diff_state.scroll_offset += 3;
        } else if in_rect(&layout.sidebar_rect) {
            self.sidebar_state.scroll_offset += 3;
        }
    }

    fn handle_mouse_scroll_down(&mut self, layout: &LayoutState, in_rect: &dyn Fn(&Rect) -> bool) {
        if in_rect(&layout.chat_rect) {
            self.chat_state.scroll_offset = self.chat_state.scroll_offset.saturating_sub(3);
        } else if in_rect(&layout.diff_rect) {
            self.diff_state.scroll_offset = self.diff_state.scroll_offset.saturating_sub(3);
        } else if in_rect(&layout.sidebar_rect) {
            self.sidebar_state.scroll_offset = self.sidebar_state.scroll_offset.saturating_sub(3);
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

    // ── Agent & config events ──────────────────────────────

    /// Handle agent streaming events
    pub(crate) fn handle_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::TextDelta(text) => {
                self.chat_state.streaming_text.push_str(&text);
                self.chat_state.is_streaming = true;
            }
            AgentEvent::ToolCallStart { name, args } => {
                self.status_bar.agent_state = AgentState::RunningTool(name);
                tracing::info!(
                    "Tool call: {:?} with args: {}",
                    self.status_bar.agent_state,
                    args
                );
            }
            AgentEvent::ToolResult {
                name,
                output,
                is_error,
            } => {
                let preview_len = output
                    .char_indices()
                    .nth(100)
                    .map(|(i, _)| i)
                    .unwrap_or(output.len());
                tracing::info!(
                    "Tool result: {} (error={}): {}",
                    name,
                    is_error,
                    &output[..preview_len]
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

    /// Apply a config hot-reload
    pub(crate) fn apply_config_reload(&mut self, reload: ConfigReload) {
        tracing::info!(
            version = reload.version,
            theme = %reload.settings.ui.theme,
            keymap = %reload.settings.ui.keymap,
            sandbox_mode = ?reload.settings.sandbox.mode,
            "Applying config hot-reload"
        );

        self.keybindings = Self::create_keybindings(&reload.settings);
        self.settings.ui.theme = reload.settings.ui.theme.clone();
        self.settings.ui.keymap = reload.settings.ui.keymap.clone();
        self.settings.sandbox.mode = reload.settings.sandbox.mode.clone();
        self.status_bar.sandbox_mode = format!("{:?}", self.settings.sandbox.mode);
        self.config_version = reload.version;
    }

    // ── Drawing ────────────────────────────────────────────

    /// Draw the frame (read-only) — kept for API consistency
    #[allow(dead_code)]
    pub(crate) fn draw(&self, frame: &mut ratatui::Frame) {
        crate::tui::renderer::draw_frame(frame, self);
    }

    pub(crate) fn draw_with_layout(&mut self, frame: &mut ratatui::Frame) {
        crate::tui::renderer::draw_frame_with_layout(frame, self);
    }
}
