//! Plugin system for Synerix
//!
//! Plugins can register tools, skills, and hook into agent lifecycle events.

mod manager;
mod types;

// Re-export public API
pub use manager::PluginManager;
pub use types::{Plugin, PluginEvent};
