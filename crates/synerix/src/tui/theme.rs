//! Theme definitions — refined Tokyo Night palette with gradient support.
//!
//! ## Color system
//! - `ThemeManager` provides thread-safe runtime palette access via `Arc<RwLock<ColorPalette>>`.
//! - `read_palette()` returns a RAII guard that derefs to `ColorPalette`.
//! - `gradient_at()` does linear RGB interpolation for title bars and highlights.
//! - `gradient_border_style()` builds a gradient border style for focus indicators.
//!
//! ## Design tokens
//! | Token | Usage |
//! |---|---|
//! | `background` | Root background |
//! | `surface` | Panel backgrounds (sidebar, input, status bar) |
//! | `foreground` | Primary text |
//! | `accent` | Active elements, focus indicators |
//! | `border` | Panel borders |
//! | `error` / `warning` / `success` | Semantic feedback |
//! | `comment` | Dimmed hints, separators |
//! | `muted_fg` | Secondary text |
//! | `chat_user` / `chat_assistant` / `chat_system` / `chat_tool` | Role colors |
//! | `streaming_cursor` | Blinking cursor |
//! | `overlay` | Popup / approval backgrounds |
//! | `highlight_bg` | Selected item background |
//! | `focus_glow` | Focus halo / glow effect |
//! | `gradient_start` / `gradient_end` | Gradient title bar colors |
//! | `title_bar_bg` | Title bar background |
//! | `shadow_color` | Shadow line color |
//! | `divider_color` | Divider line color |
//! | `border_focus` | Focused panel border (distinct from `border`) |
//! | `diff_add_fg` / `diff_add_bg` | Added lines in diff |
//! | `diff_remove_fg` / `diff_remove_bg` | Removed lines in diff |

use ratatui::style::{Color, Style};
use ratatui::widgets::BorderType;

// ── Color palette ────────────────────────────────────────────────────────

/// Complete color palette for a theme.
#[derive(Clone, Copy, Debug)]
pub struct ColorPalette {
    // Base layers
    pub background: Color,
    pub surface: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    // Semantic
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub comment: Color,
    pub muted_fg: Color,
    // Chat roles
    pub chat_user: Color,
    pub chat_assistant: Color,
    pub chat_system: Color,
    pub chat_tool: Color,
    // Streaming
    pub streaming_cursor: Color,
    // Overlays & highlights
    pub overlay: Color,
    pub highlight_bg: Color,
    // Focus & visual polish
    pub focus_glow: Color,
    pub gradient_start: Color,
    pub gradient_end: Color,
    pub title_bar_bg: Color,
    pub shadow_color: Color,
    pub divider_color: Color,
    pub border_focus: Color,
    // Diff colors
    pub diff_add_fg: Color,
    pub diff_add_bg: Color,
    pub diff_remove_fg: Color,
    pub diff_remove_bg: Color,
}

// ── RAII read guard ──────────────────────────────────────────────────────

/// RAII guard for reading the current palette.
///
/// Derefs to `ColorPalette`, so all existing field access (`p.accent`, etc.)
/// continues to work without modification.
#[derive(Clone, Copy, Debug)]
pub struct ThemeReadGuard {
    palette: ColorPalette,
}

impl std::ops::Deref for ThemeReadGuard {
    type Target = ColorPalette;
    fn deref(&self) -> &Self::Target {
        &self.palette
    }
}

// ── Theme manager ────────────────────────────────────────────────────────

static THEME_MANAGER: once_cell::sync::Lazy<std::sync::Mutex<ColorPalette>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(dark_theme()));

/// Initialise the theme palette (called once at startup).
pub fn init_theme(dark: bool) {
    let palette = if dark { dark_theme() } else { light_theme() };
    let mut guard = THEME_MANAGER.lock().unwrap();
    *guard = palette;
}

/// Get a read guard for the current palette.
///
/// Returns a guard that derefs to `ColorPalette`.  The palette is cloned
/// internally (cheap — `ColorPalette` is ~30 bytes), so the guard owns its
/// data and imposes no lifetime constraints on callers.
pub fn read_palette() -> ThemeReadGuard {
    let guard = THEME_MANAGER.lock().unwrap();
    ThemeReadGuard {
        palette: (*guard).clone(),
    }
}

/// Execute `f` with a read guard for the current palette.
///
/// Convenience helper that avoids explicit guard management:
/// ```rust
/// # synerix::tui::theme::init_theme(true);
/// synerix::tui::theme::with_palette(|p| {
///     println!("accent = {:?}", p.accent);
/// });
/// ```
pub fn with_palette<F: FnOnce(&ColorPalette)>(f: F) {
    let guard = read_palette();
    f(&guard);
}

/// Runtime theme manager for hot-reload support.
#[derive(Clone)]
pub struct ThemeManager {
    inner: std::sync::Arc<std::sync::Mutex<ColorPalette>>,
}

impl ThemeManager {
    /// Create a new manager from an existing palette.
    pub fn new(palette: ColorPalette) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(palette)),
        }
    }

    /// Swap in a new palette (hot-reload).
    pub fn set(&self, palette: ColorPalette) {
        let mut guard = self.inner.lock().unwrap();
        *guard = palette;
    }

    /// Read the current palette.
    pub fn current(&self) -> ThemeReadGuard {
        let guard = self.inner.lock().unwrap();
        ThemeReadGuard {
            palette: (*guard).clone(),
        }
    }

    /// Execute `f` with a read guard.
    pub fn with<F: FnOnce(&ColorPalette)>(&self, f: F) {
        let guard = self.current();
        f(&guard);
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(dark_theme())
    }
}

/// Access the global theme singleton.
pub fn global_manager() -> &'static ThemeManager {
    static GLOBAL: once_cell::sync::Lazy<ThemeManager> =
        once_cell::sync::Lazy::new(ThemeManager::default);
    &GLOBAL
}

// ── Gradient utilities ───────────────────────────────────────────────────

/// A linear color gradient between two colors.
#[derive(Clone, Debug)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
    pub steps: usize,
}

impl Gradient {
    pub fn new(start: Color, end: Color, steps: usize) -> Self {
        Self { start, end, steps }
    }

    /// Return the color at position `i` (0-based) out of `total` steps.
    pub fn at(&self, i: usize, total: usize) -> Color {
        gradient_at(self.start, self.end, i, total)
    }
}

/// Linear RGB interpolation between two colors.
///
/// Returns the color at position `step` out of `total` steps.
/// When `total <= 1`, returns `color1`.
pub fn gradient_at(color1: Color, color2: Color, step: usize, total: usize) -> Color {
    if total <= 1 {
        return color1;
    }
    let t = step as f64 / (total - 1) as f64;
    let (r1, g1, b1) = rgb_components(color1);
    let (r2, g2, b2) = rgb_components(color2);
    let r = (r1 as f64 + t * (r2 as f64 - r1 as f64)).round() as u8;
    let g = (g1 as f64 + t * (g2 as f64 - g1 as f64)).round() as u8;
    let b = (b1 as f64 + t * (b2 as f64 - b1 as f64)).round() as u8;
    Color::Rgb(r, g, b)
}

fn rgb_components(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    }
}

// ── Cached gradient separator ──────────────────────────────────────────────

use std::sync::Mutex;

/// Cache for header gradient separator spans, keyed by terminal width.
/// The separator rarely changes (only on terminal resize), so caching avoids
/// rebuilding `width` spans every animated frame.
static SEP_CACHE: once_cell::sync::Lazy<Mutex<Vec<(u16, Vec<ratatui::text::Span<'static>>)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Vec::new()));

/// Get or build a gradient separator for the given width.
/// Returns a Vec of single-character `─` spans with gradient colors.
/// Cached: only rebuilds when width changes.
pub fn cached_gradient_separator(width: u16) -> Vec<ratatui::text::Span<'static>> {
    use ratatui::text::Span;
    use ratatui::style::Style;

    // Fast path: check cache for matching width
    {
        let cache = SEP_CACHE.lock().unwrap();
        if let Some(entry) = cache.iter().find(|(w, _)| *w == width) {
            return entry.1.clone();
        }
    }

    // Build separator for this width
    let palette = read_palette();
    let gradient = Gradient::new(palette.gradient_start, palette.gradient_end, width as usize);
    let mut spans = Vec::with_capacity(width as usize);
    for i in 0..width {
        let color = gradient.at(i as usize, width as usize);
        spans.push(Span::styled("─", Style::default().fg(color)));
    }

    // Cache it (limit cache to 10 entries to avoid memory growth)
    let mut cache = SEP_CACHE.lock().unwrap();
    if cache.len() >= 10 {
        cache.clear();
    }
    cache.push((width, spans.clone()));

    spans
}

// ── Border helpers ───────────────────────────────────────────────────────

/// Rounded border type — used everywhere for a modern look.
pub const BORDER_TYPE: BorderType = BorderType::Rounded;

/// Standard color constants (independent of theme, always the same)
pub const COLOR_DARK_GRAY: Color = Color::DarkGray;
pub const COLOR_GRAY: Color = Color::Gray;
pub const COLOR_CYAN: Color = Color::Cyan;

/// Build a border style using the theme's border color and rounded type.
pub fn border_style(p: &ColorPalette) -> Style {
    Style::default().fg(p.border)
}

/// Build a focused border style using the theme's border_focus color.
pub fn border_focus_style(p: &ColorPalette) -> Style {
    Style::default().fg(p.border_focus)
}

/// Muted / dim text style
pub fn muted_style() -> Style {
    Style::default().fg(COLOR_DARK_GRAY)
}

/// Build a gradient border style interpolating between two colors over `width` cells.
///
/// Returns a `Style` — ratatui does not support per-cell border colors natively,
/// so this interpolates the midpoint color for a subtle two-tone effect.
pub fn gradient_border_style(p: &ColorPalette, width: u16) -> Style {
    if width <= 1 {
        return Style::default().fg(p.gradient_start);
    }
    let mid = gradient_at(p.gradient_start, p.gradient_end, (width / 2) as usize, width as usize);
    Style::default().fg(mid)
}

// ── Shadow helpers ───────────────────────────────────────────────────────

/// Shadow line character — Unicode lower half block.
pub const SHADOW_CHAR: &str = "▄";

// ── Table separators (Codex pattern) ───────────────────────────────────────

/// Heavy separator for table header rows (`━`).
pub const TABLE_SEP_HEADER: &str = "━";
/// Light separator for table body rows (`─`).
pub const TABLE_SEP_BODY: &str = "─";
/// Column junction character (`┼`).
pub const TABLE_SEP_JUNCTION: &str = "┼";

// ── Dark theme (refined Tokyo Night) ─────────────────────────────────────

/// Dark theme — refined Tokyo Night with deeper surfaces and softer accent.
pub fn dark_theme() -> ColorPalette {
    ColorPalette {
        // Base layers
        background: Color::Rgb(22, 24, 35),
        surface: Color::Rgb(30, 32, 48),
        foreground: Color::Rgb(192, 202, 245),
        accent: Color::Rgb(137, 180, 250),
        border: Color::Rgb(49, 50, 68),
        // Semantic
        error: Color::Rgb(247, 118, 142),
        warning: Color::Rgb(224, 175, 104),
        success: Color::Rgb(158, 206, 121),
        comment: Color::Rgb(120, 126, 160),
        muted_fg: Color::Rgb(152, 158, 192),
        // Chat roles
        chat_user: Color::Rgb(137, 180, 250),
        chat_assistant: Color::Rgb(158, 206, 121),
        chat_system: Color::Rgb(224, 175, 104),
        chat_tool: Color::Rgb(187, 154, 247),
        // Streaming
        streaming_cursor: Color::Rgb(137, 180, 250),
        // Overlays & highlights
        overlay: Color::Rgb(30, 32, 48),
        highlight_bg: Color::Rgb(44, 48, 72),
        // Focus & visual polish
        focus_glow: Color::Rgb(137, 180, 250),
        gradient_start: Color::Rgb(137, 180, 250),
        gradient_end: Color::Rgb(30, 32, 48),
        title_bar_bg: Color::Rgb(26, 28, 40),
        shadow_color: Color::Rgb(49, 50, 68),
        divider_color: Color::Rgb(44, 48, 72),
        border_focus: Color::Rgb(137, 180, 250),
        // Diff colors
        diff_add_fg: Color::Rgb(120, 230, 120),
        diff_add_bg: Color::Rgb(28, 48, 28),
        diff_remove_fg: Color::Rgb(230, 120, 120),
        diff_remove_bg: Color::Rgb(48, 28, 28),
    }
}

// ── Light theme ──────────────────────────────────────────────────────────

/// Light theme — clean high-contrast palette.
pub fn light_theme() -> ColorPalette {
    ColorPalette {
        // Base layers
        background: Color::Rgb(252, 252, 250),
        surface: Color::Rgb(240, 240, 238),
        foreground: Color::Rgb(52, 59, 88),
        accent: Color::Rgb(47, 128, 200),
        border: Color::Rgb(200, 202, 210),
        // Semantic
        error: Color::Rgb(206, 60, 80),
        warning: Color::Rgb(150, 89, 59),
        success: Color::Rgb(80, 160, 80),
        comment: Color::Rgb(120, 124, 140),
        muted_fg: Color::Rgb(100, 104, 120),
        // Chat roles
        chat_user: Color::Rgb(47, 128, 200),
        chat_assistant: Color::Rgb(80, 160, 80),
        chat_system: Color::Rgb(150, 89, 59),
        chat_tool: Color::Rgb(140, 100, 200),
        // Streaming
        streaming_cursor: Color::Rgb(47, 128, 200),
        // Overlays & highlights
        overlay: Color::Rgb(240, 240, 238),
        highlight_bg: Color::Rgb(220, 224, 232),
        // Focus & visual polish
        focus_glow: Color::Rgb(47, 128, 200),
        gradient_start: Color::Rgb(47, 128, 200),
        gradient_end: Color::Rgb(240, 240, 238),
        title_bar_bg: Color::Rgb(245, 246, 248),
        shadow_color: Color::Rgb(200, 202, 210),
        divider_color: Color::Rgb(220, 224, 232),
        border_focus: Color::Rgb(47, 128, 200),
        // Diff colors
        diff_add_fg: Color::Rgb(30, 130, 30),
        diff_add_bg: Color::Rgb(220, 240, 220),
        diff_remove_fg: Color::Rgb(180, 50, 50),
        diff_remove_bg: Color::Rgb(240, 220, 220),
    }
}
