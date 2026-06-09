//! Plugin system for Synerix
//!
//! Plugins can register tools, skills, and hook into agent lifecycle events.

mod manager;
#[cfg(test)]
pub mod stubs;
mod types;

// Re-export public API
pub use manager::PluginManager;
pub use types::{Plugin, PluginEvent};
