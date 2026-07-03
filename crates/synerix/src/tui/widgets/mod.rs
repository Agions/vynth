//! Composable TUI widget library

pub mod approval_popup;
pub mod chat_area;
pub mod diff_view;
pub mod input_box;
pub mod primitives;
pub mod sidebar;
pub mod slash_menu;
pub mod status_bar;

// Re-export the Widget trait and RenderContext so consumers can write
// `use crate::tui::widgets::Widget;` instead of reaching into primitives.
pub use primitives::{RenderContext, Widget};
