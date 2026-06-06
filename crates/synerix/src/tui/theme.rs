//! Theme definitions — color system and helpers for all widgets.
//! Tokyo Night inspired dark theme with a light theme counterpart.
//! Use `current_palette()` to get the active palette.

use ratatui::style::{Color, Style};
use ratatui::widgets::BorderType;
use std::sync::OnceLock;

/// Color palette for a theme
#[derive(Clone, Debug)]
pub struct ColorPalette {
    // Base colors
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub comment: Color,
    // Chat role colors
    pub chat_user: Color,
    pub chat_assistant: Color,
    pub chat_system: Color,
    pub chat_tool: Color,
    // Streaming cursor
    pub streaming_cursor: Color,
    // Text style colors
    pub muted_fg: Color,
}

// ── Singleton global theme ──────────────────────────────────────────────

static CURRENT_THEME: OnceLock<ColorPalette> = OnceLock::new();

/// Initialise the theme palette (called once at startup).
pub fn init_theme(dark: bool) {
    let _ = CURRENT_THEME.set(if dark { dark_theme() } else { light_theme() });
}

/// Get the current active palette (panics if not yet initialised).
pub fn current_palette() -> &'static ColorPalette {
    CURRENT_THEME.get().expect("theme not initialised — call init_theme() first")
}

// ── Border helpers ──────────────────────────────────────────────────────

/// Rounded border type — used everywhere for a modern look.
pub const BORDER_TYPE: BorderType = BorderType::Rounded;

/// Standard color constants (independent of theme, always the same)
pub const COLOR_DARK_GRAY: Color = Color::DarkGray;
pub const COLOR_GRAY: Color = Color::Gray;
pub const COLOR_CYAN: Color = Color::Cyan;

/// Muted / dim text style
pub fn muted_style() -> Style {
    Style::default().fg(COLOR_DARK_GRAY)
}

/// Dark theme (default: Tokyo Night inspired)
pub fn dark_theme() -> ColorPalette {
    ColorPalette {
        background: Color::Rgb(26, 27, 38),
        foreground: Color::Rgb(192, 202, 245),
        accent: Color::Rgb(125, 207, 255),
        border: Color::Rgb(59, 62, 84),
        error: Color::Rgb(247, 118, 142),
        warning: Color::Rgb(224, 175, 104),
        success: Color::Rgb(158, 206, 121),
        comment: Color::Rgb(86, 92, 116),
        chat_user: Color::Rgb(125, 207, 255),
        chat_assistant: Color::Rgb(158, 206, 121),
        chat_system: Color::Rgb(224, 175, 104),
        chat_tool: Color::Rgb(187, 154, 247),
        streaming_cursor: Color::Rgb(125, 207, 255),
        muted_fg: Color::Rgb(105, 112, 140),
    }
}

/// Light theme
#[allow(dead_code)]
pub fn light_theme() -> ColorPalette {
    ColorPalette {
        background: Color::Rgb(252, 252, 250),
        foreground: Color::Rgb(52, 59, 88),
        accent: Color::Rgb(47, 128, 200),
        border: Color::Rgb(200, 202, 210),
        error: Color::Rgb(206, 60, 80),
        warning: Color::Rgb(150, 89, 59),
        success: Color::Rgb(80, 160, 80),
        comment: Color::Rgb(150, 152, 160),
        chat_user: Color::Rgb(47, 128, 200),
        chat_assistant: Color::Rgb(80, 160, 80),
        chat_system: Color::Rgb(150, 89, 59),
        chat_tool: Color::Rgb(140, 100, 200),
        streaming_cursor: Color::Rgb(47, 128, 200),
        muted_fg: Color::Rgb(140, 142, 155),
    }
}