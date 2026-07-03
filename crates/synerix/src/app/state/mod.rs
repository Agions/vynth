//! Application state types — split into domain sub-modules.
//!
//! Sub-modules:
//! - `layout` — FocusedPanel, LayoutState
//! - `chat` — ChatState
//! - `sidebar` — SidebarState, SidebarTab, FileEntry
//! - `diff` — DiffState, DiffHunk, DiffLine, DiffLineKind
//! - `status` — StatusBarState, AgentState
//! - `goal` — GoalState
//! - `dirty` — DirtyFlags
//! - `input` — InputMode
//! - `coding_mode` — CodingMode
//! - `app` — App struct + constructors + methods

mod app;
mod chat;
mod coding_mode;
mod diff;
mod dirty;
mod goal;
mod input;
mod layout;
mod sidebar;
mod slash_menu_state;
mod status;

pub use app::App;
pub use chat::ChatState;
pub use coding_mode::CodingMode;
pub use diff::{DiffHunk, DiffLine, DiffLineKind, DiffState};
pub use dirty::DirtyFlags;
pub use goal::GoalState;
pub use input::InputMode;
pub use layout::{FocusedPanel, LayoutState};
pub use sidebar::{FileEntry, SidebarState, SidebarTab};
pub use slash_menu_state::SlashMenuState;
pub use status::{AgentState, StatusBarState};
