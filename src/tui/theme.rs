//! Theme definitions

use ratatui::style::Color;

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
    }
}

/// Light theme
pub fn light_theme() -> ColorPalette {
    ColorPalette {
        background: Color::Rgb(252, 252, 252),
        foreground: Color::Rgb(59, 60, 68),
        accent: Color::Rgb(47, 130, 200),
        border: Color::Rgb(200, 200, 200),
        error: Color::Rgb(206, 60, 80),
        warning: Color::Rgb(180, 130, 50),
        success: Color::Rgb(80, 160, 80),
        comment: Color::Rgb(150, 150, 150),
        // Chat
        chat_user: Color::Rgb(47, 130, 200),
        chat_assistant: Color::Rgb(80, 160, 80),
        chat_system: Color::Rgb(180, 130, 50),
        chat_tool: Color::Rgb(140, 100, 200),
        // Sidebar
        sidebar_bg: Color::Rgb(245, 245, 245),
        sidebar_fg: Color::Rgb(80, 80, 90),
        sidebar_active: Color::Rgb(47, 130, 200),
        // Input
        input_bg: Color::Rgb(255, 255, 255),
        // Diff
        diff_add: Color::Rgb(80, 160, 80),
        diff_remove: Color::Rgb(206, 60, 80),
        diff_header: Color::Rgb(47, 130, 200),
        // Status
        status_bg: Color::Rgb(230, 230, 230),
        status_fg: Color::Rgb(59, 60, 68),
        // Streaming
        streaming_cursor: Color::Rgb(47, 130, 200),
    }
}
