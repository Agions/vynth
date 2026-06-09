//! Default value functions for config types.
//!
//! Extracted from `mod.rs` to keep type definitions clean.
//! Used by `Settings::defaults()` (in `validation.rs`).

use super::SandboxMode;

pub(crate) fn default_system_prompt_tokens() -> usize {
    2000
}

pub(crate) fn default_tools_schema_tokens() -> usize {
    3000
}

pub(crate) fn default_temperature() -> f32 {
    0.7
}

pub(crate) fn default_theme() -> String {
    "dark".to_string()
}

pub(crate) fn default_keymap() -> String {
    "default".to_string()
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_typing_delay() -> u64 {
    10
}

pub(crate) fn default_tool_timeout() -> u64 {
    120
}

pub(crate) fn default_sandbox_mode() -> SandboxMode {
    SandboxMode::Confirm
}

pub(crate) fn default_mcp_timeout() -> u64 {
    30
}

pub(crate) fn default_agent_turns() -> usize {
    10
}
