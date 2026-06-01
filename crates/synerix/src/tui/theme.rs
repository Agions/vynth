//! Theme definitions

use ratatui::style::{Color, Modifier, Style};

/// Color palette for a theme
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
    // Sidebar colors
    pub sidebar_bg: Color,
    pub sidebar_fg: Color,
    pub sidebar_active: Color,
    // Input colors
    pub input_bg: Color,
    // Diff colors
    pub diff_add: Color,
    pub diff_remove: Color,
    pub diff_header: Color,
    // Status bar colors
    pub status_bg: Color,
    pub status_fg: Color,
    // Streaming cursor
    pub streaming_cursor: Color,
    // Selection colors
    pub selection_bg: Color,
    pub selection_fg: Color,
    // Highlight colors
    pub highlight_bg: Color,
    // Text style colors
    pub muted_fg: Color,
    pub link_fg: Color,
    pub code_bg: Color,
    pub quote_fg: Color,
    pub separator_fg: Color,
}

/// Standard color constants (independent of theme, always the same)
pub const COLOR_DARK_GRAY: Color = Color::DarkGray;
pub const COLOR_GRAY: Color = Color::Gray;
pub const COLOR_WHITE: Color = Color::White;
pub const COLOR_BLACK: Color = Color::Black;
pub const COLOR_CYAN: Color = Color::Cyan;
pub const COLOR_GREEN: Color = Color::Green;
pub const COLOR_YELLOW: Color = Color::Yellow;
pub const COLOR_MAGENTA: Color = Color::Magenta;
pub const COLOR_RED: Color = Color::Red;
pub const COLOR_BLUE: Color = Color::Blue;

/// Create border style for a panel — focused vs unfocused
pub fn panel_border_style(palette: &ColorPalette, is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(palette.accent)
    } else {
        Style::default().fg(COLOR_DARK_GRAY)
    }
}

/// Status bar background style
pub fn status_bg_style(palette: &ColorPalette) -> Style {
    Style::default().bg(palette.status_bg).fg(palette.status_fg)
}

/// Role name style
pub fn role_style(palette: &ColorPalette, role_name: &str) -> Style {
    let color = match role_name {
        "user" | "User" => palette.chat_user,
        "assistant" | "Assistant" => palette.chat_assistant,
        "system" | "System" => palette.chat_system,
        "tool" | "Tool" => palette.chat_tool,
        _ => palette.foreground,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

/// Muted / dim text style
pub fn muted_style() -> Style {
    Style::default().fg(COLOR_DARK_GRAY)
}

/// Error text style
pub fn error_style(palette: &ColorPalette) -> Style {
    Style::default().fg(palette.error)
}

/// Success text style
pub fn success_style(palette: &ColorPalette) -> Style {
    Style::default().fg(palette.success)
}

/// Link style
pub fn link_style(palette: &ColorPalette) -> Style {
    Style::default().fg(palette.link_fg)
}

/// Available themes
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    /// Resolve a theme variant to its concrete `ColorPalette`.
    pub fn resolve(&self) -> ColorPalette {
        match self {
            Theme::Dark => dark_theme(),
            Theme::Light => light_theme(),
        }
    }
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
        // Chat
        chat_user: Color::Rgb(125, 207, 255),
        chat_assistant: Color::Rgb(158, 206, 121),
        chat_system: Color::Rgb(224, 175, 104),
        chat_tool: Color::Rgb(187, 154, 247),
        // Sidebar
        sidebar_bg: Color::Rgb(22, 23, 32),
        sidebar_fg: Color::Rgb(160, 170, 210),
        sidebar_active: Color::Rgb(125, 207, 255),
        // Input
        input_bg: Color::Rgb(30, 32, 48),
        // Diff
        diff_add: Color::Rgb(158, 206, 121),
        diff_remove: Color::Rgb(247, 118, 142),
        diff_header: Color::Rgb(125, 207, 255),
        // Status
        status_bg: Color::Rgb(40, 42, 58),
        status_fg: Color::Rgb(192, 202, 245),
        // Streaming
        streaming_cursor: Color::Rgb(125, 207, 255),
        // Selection
        selection_bg: Color::Rgb(65, 68, 100),
        selection_fg: Color::Rgb(224, 227, 255),
        // Highlight
        highlight_bg: Color::Rgb(224, 175, 104),
        // Text styles
        muted_fg: Color::Rgb(105, 112, 140),
        link_fg: Color::Rgb(125, 207, 255),
        code_bg: Color::Rgb(35, 37, 52),
        quote_fg: Color::Rgb(140, 148, 180),
        separator_fg: Color::Rgb(50, 53, 72),
    }
}

/// Light theme
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
        // Chat
        chat_user: Color::Rgb(47, 128, 200),
        chat_assistant: Color::Rgb(80, 160, 80),
        chat_system: Color::Rgb(150, 89, 59),
        chat_tool: Color::Rgb(140, 100, 200),
        // Sidebar
        sidebar_bg: Color::Rgb(245, 245, 242),
        sidebar_fg: Color::Rgb(80, 80, 90),
        sidebar_active: Color::Rgb(47, 128, 200),
        // Input
        input_bg: Color::Rgb(255, 255, 255),
        // Diff
        diff_add: Color::Rgb(80, 160, 80),
        diff_remove: Color::Rgb(206, 60, 80),
        diff_header: Color::Rgb(47, 128, 200),
        // Status
        status_bg: Color::Rgb(230, 230, 228),
        status_fg: Color::Rgb(52, 59, 88),
        // Streaming
        streaming_cursor: Color::Rgb(47, 128, 200),
        // Selection
        selection_bg: Color::Rgb(190, 210, 240),
        selection_fg: Color::Rgb(20, 30, 60),
        // Highlight
        highlight_bg: Color::Rgb(255, 230, 150),
        // Text styles
        muted_fg: Color::Rgb(140, 142, 155),
        link_fg: Color::Rgb(47, 128, 200),
        code_bg: Color::Rgb(238, 238, 235),
        quote_fg: Color::Rgb(100, 105, 120),
        separator_fg: Color::Rgb(210, 212, 220),
    }
}
