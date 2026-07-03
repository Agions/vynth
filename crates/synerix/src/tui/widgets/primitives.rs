//! Shared widget primitives and the unified `Widget` trait.
//!
//! ## Architecture
//!
//! All TUI widgets implement the [`Widget`] trait, which standardises the
//! render signature and encourages composition via [`RenderContext`].
//!
//! ### Primitive widgets
//! - [`Panel`] — bordered container with optional gradient title bar.
//! - [`Divider`] — horizontal separator (heavy / medium / light).
//! - [`ShadowLine`] — bottom shadow effect for depth.
//! - [`FocusRing`] — focused-panel border glow indicator.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{CodingMode, FocusedPanel, InputMode};
use crate::config::SandboxMode;
use crate::tui::theme;

// ── Render context ─────────────────────────────────────────────────────

/// Snapshot of app state needed for a single render pass.
///
/// Constructed once per frame in [`crate::tui::renderer`] and passed to
/// every widget.  Keeps widget signatures clean and makes testing trivial
/// (no need to construct a full `App`).
#[derive(Clone, Debug)]
pub struct RenderContext<'a> {
    // Theme — owned guard (ThemeReadGuard derefs to ColorPalette)
    pub palette: theme::ThemeReadGuard,
    // Animation
    pub anim_frame: u64,
    pub is_streaming: bool,
    // Focus
    pub focused: FocusedPanel,
    // Modes & config
    pub coding_mode: CodingMode,
    pub input_mode: InputMode,
    pub sandbox_mode: SandboxMode,
    // Input
    pub input_buffer: &'a str,
    pub input_cursor: usize,
    // Chat
    pub chat_messages: &'a [crate::app::message::ChatMessage],
    pub streaming_text: &'a str,
    pub scroll_offset: usize,
    // Pre-computed header gradient separator (cached per render call, width = terminal cols)
    pub header_separator: Vec<ratatui::text::Span<'static>>,
    // Status bar
    pub model_name: &'a str,
    pub tokens_used: usize,
    pub tokens_total: usize,
    pub goal_active: bool,
    pub goal_duration: &'a str,
    pub agent_state: &'a crate::app::AgentState,
    pub agent_start: Option<std::time::Instant>,
    // Sidebar
    pub sidebar_tab: crate::app::SidebarTab,
    pub sidebar_file_tree: &'a [crate::app::FileEntry],
    pub sidebar_scroll: usize,
    // Diff
    pub diff_content: &'a str,
    pub diff_scroll: usize,
    // Approval
    pub approval_pending: bool,
    pub approval_text: Option<&'a str>,
    // Slash
    pub slash_selected: usize,
    // Config
    pub api_key_configured: bool,
}

impl<'a> RenderContext<'a> {
    /// Build a context from live app state.
    pub fn from_app(app: &'a crate::app::App) -> Self {
        Self {
            palette: crate::tui::theme::read_palette(),
            anim_frame: app.status_bar.animation_frame,
            is_streaming: app.chat_state.is_streaming,
            focused: app.focused_panel.clone(),
            coding_mode: app.coding_mode,
            input_mode: app.mode,
            sandbox_mode: app.settings.sandbox.mode.clone(),
            input_buffer: &app.input_buffer,
            input_cursor: app.input_cursor,
            chat_messages: &app.chat_state.messages,
            streaming_text: &app.chat_state.streaming_text,
            scroll_offset: app.chat_state.scroll_offset,
            model_name: &app.status_bar.model_name,
            tokens_used: app.status_bar.tokens_used,
            tokens_total: app.status_bar.tokens_total,
            goal_active: app.status_bar.goal_active,
            goal_duration: &app.status_bar.goal_duration,
            agent_state: &app.status_bar.agent_state,
            agent_start: app.status_bar.agent_start_time,
            sidebar_tab: app.sidebar_state.active_tab,
            sidebar_file_tree: &app.sidebar_state.file_tree,
            sidebar_scroll: app.sidebar_state.scroll_offset,
            diff_content: &app.diff_state.content,
            diff_scroll: app.diff_state.scroll_offset,
            approval_pending: app.pending_approval.is_some(),
            approval_text: app.pending_approval.as_deref(),
            slash_selected: app.slash_menu_state.selected,
            api_key_configured: !app.settings.llm.api_key.trim().is_empty(),
            header_separator: Vec::new(),
        }
    }

    /// Check whether `panel` is the focused panel.
    pub fn is_focused(&self, panel: FocusedPanel) -> bool {
        std::mem::discriminant(&self.focused) == std::mem::discriminant(&panel)
    }
}

// ── Widget trait ──────────────────────────────────────────────────────

/// Unified widget render trait.
///
/// Every renderable component in the TUI implements this.  The renderer
/// iterates over registered widgets and calls [`Self::render`] for each
/// dirty region.
pub trait Widget {
    /// Render into `area` on `frame` using `ctx`.
    fn render(&self, area: Rect, frame: &mut Frame, ctx: &RenderContext);
}

// ── Primitive: Panel ───────────────────────────────────────────────────

/// A bordered panel container with optional gradient title bar.
#[derive(Clone, Debug, Default)]
pub struct Panel<'a> {
    /// Title text (shown in the top border).
    pub title: &'a str,
    /// Border style override (defaults to theme border color).
    pub border_style: Option<Style>,
    /// Whether the panel is focused (uses `border_focus` color).
    pub focused: bool,
    /// Whether to render a gradient-filled title bar background.
    pub gradient_title: bool,
}

impl<'a> Panel<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            ..Self::default()
        }
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_gradient_title(mut self, yes: bool) -> Self {
        self.gradient_title = yes;
        self
    }

    pub fn with_border_style(mut self, style: Style) -> Self {
        self.border_style = Some(style);
        self
    }
}

impl<'a> Widget for Panel<'a> {
    fn render(&self, area: Rect, frame: &mut Frame, ctx: &RenderContext) {
        let p = ctx.palette;
        let border = self.border_style.unwrap_or_else(|| {
            if self.focused {
                Style::default().fg(p.border_focus).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.border)
            }
        });

        let title_style = if self.focused {
            Style::default()
                .fg(p.background)
                .bg(p.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
        };

        let block = Block::default()
            .title(Span::styled(self.title, title_style))
            .borders(Borders::ALL)
            .border_type(theme::BORDER_TYPE)
            .border_style(border);

        frame.render_widget(block, area);
    }
}

// ── Primitive: Divider ─────────────────────────────────────────────────

/// Horizontal separator weight.
#[derive(Clone, Copy, Debug, Default)]
pub enum DividerWeight {
    #[default]
    Medium,
    Heavy,
    Light,
}

/// A horizontal divider line.
#[derive(Clone, Debug, Default)]
pub struct Divider {
    pub weight: DividerWeight,
}

impl Divider {
    pub fn new(weight: DividerWeight) -> Self {
        Self { weight }
    }

    fn char(&self) -> &'static str {
        match self.weight {
            DividerWeight::Heavy => theme::TABLE_SEP_HEADER,
            DividerWeight::Medium => "─",
            DividerWeight::Light => theme::TABLE_SEP_BODY,
        }
    }
}

impl Widget for Divider {
    fn render(&self, area: Rect, frame: &mut Frame, ctx: &RenderContext) {
        let line = Line::from(Span::styled(
            self.char().repeat(area.width as usize),
            Style::default().fg(ctx.palette.divider_color),
        ));
        frame.render_widget(Paragraph::new(line), area);
    }
}

// ── Primitive: ShadowLine ──────────────────────────────────────────────

/// Bottom shadow line for depth effect.
#[derive(Clone, Debug, Default)]
pub struct ShadowLine;

impl Widget for ShadowLine {
    fn render(&self, area: Rect, frame: &mut Frame, ctx: &RenderContext) {
        if area.height == 0 {
            return;
        }
        let y = area.y + area.height - 1;
        let x = area.x + 1;
        let w = area.width.saturating_sub(2);
        if w == 0 {
            return;
        }
        let rect = Rect::new(x, y, w, 1);
        let line = Line::from(Span::styled(
            theme::SHADOW_CHAR.repeat(w as usize),
            Style::default().fg(ctx.palette.shadow_color),
        ));
        frame.render_widget(Paragraph::new(line), rect);
    }
}

// ── Primitive: FocusRing ───────────────────────────────────────────────

/// Focus ring — draws a glowing border on the focused panel.
#[derive(Clone, Debug, Default)]
pub struct FocusRing {
    pub panel: FocusedPanel,
    /// Alpha multiplier for the glow (0.0 = transparent, 1.0 = full).
    pub alpha: f64,
}

impl FocusRing {
    pub fn new(panel: FocusedPanel) -> Self {
        Self { panel, alpha: 1.0 }
    }

    /// Set the glow alpha intensity (0.0–1.0).
    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Apply alpha to a color by blending toward the background color.
    fn alpha_color(&self, fg: Color, bg: Color, alpha: f64) -> Color {
        if alpha >= 1.0 {
            return fg;
        }
        if alpha <= 0.0 {
            return bg;
        }
        // Extract RGB components
        let (fr, fg_c, fb) = match fg {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => return fg,
        };
        let (br, bg_c, bb) = match bg {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => return fg,
        };
        let r = (br as f64 + alpha * (fr as f64 - br as f64)).round() as u8;
        let g = (bg_c as f64 + alpha * (fg_c as f64 - bg_c as f64)).round() as u8;
        let b = (bb as f64 + alpha * (fb as f64 - bb as f64)).round() as u8;
        Color::Rgb(r, g, b)
    }
}

impl Widget for FocusRing {
    fn render(&self, area: Rect, frame: &mut Frame, ctx: &RenderContext) {
        if !ctx.is_focused(self.panel) || area.height < 2 || area.width < 2 {
            return;
        }
        let p = ctx.palette;
        let inner = area.inner(ratatui::layout::Margin::new(1, 1));
        if inner.height == 0 || inner.width == 0 {
            return;
        }
        let glow_bg = self.alpha_color(p.focus_glow, p.background, self.alpha);
        let glow = Style::default().bg(glow_bg);
        let top = Rect::new(inner.x, inner.y - 1, inner.width, 1);
        let bottom = Rect::new(inner.x, inner.y + inner.height, inner.width, 1);
        let left = Rect::new(inner.x - 1, inner.y, 1, inner.height);
        let right = Rect::new(inner.x + inner.width, inner.y, 1, inner.height);
        for rect in [top, bottom, left, right] {
            if rect.width > 0 && rect.height > 0 {
                let cell = Paragraph::new(Line::from(Span::styled(
                    " ".repeat(rect.width as usize),
                    glow,
                )));
                frame.render_widget(cell, rect);
            }
        }
    }
}
