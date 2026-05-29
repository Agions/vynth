//! Theme definitions

use ratatui::style::Color;

/// Color palette for a theme
pub struct ColorPalette {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub comment: Color,
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
    }
}
