//! Plugin types: events and the core Plugin trait.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::AppError;
use crate::skills::SkillDef;
use crate::tools::Tool;

// ---------------------------------------------------------------------------
// PluginEvent — lifecycle events a plugin can observe
// ---------------------------------------------------------------------------

/// Events emitted during agent execution that plugins can react to.
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Emitted just before a tool is executed.
    PreToolCall { tool_name: String, args: Value },
    /// Emitted after a tool finishes execution.
    PostToolCall {
        tool_name: String,
        args: Value,
        output: String,
        is_error: bool,
    },
    /// Emitted at the start of an agent turn (before LLM call).
    PreAgentTurn { turn_number: usize },
    /// Emitted at the end of an agent turn (after LLM response processed).
    PostAgentTurn { turn_number: usize },
    /// Emitted when a workflow step completes.
    WorkflowStepComplete { step_id: String, success: bool },
    /// Custom event with an arbitrary name and payload for user-defined hooks.
    Custom { name: String, payload: Value },
}

// ---------------------------------------------------------------------------
// Plugin trait
// ---------------------------------------------------------------------------

/// Core plugin interface.
///
/// Each plugin must be `Send + Sync` so it can be shared across async tasks.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Unique plugin name (e.g. `"git-helper"`).
    fn name(&self) -> &str;

    /// Semver version string (e.g. `"0.1.0"`).
    fn version(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// One-time initialisation hook called after registration.
    /// Default implementation is a no-op.
    async fn init(&mut self) -> Result<(), AppError> {
        Ok(())
    }

    /// Tools contributed by this plugin.
    /// Default returns an empty vec.
    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        Vec::new()
    }

    /// Skills contributed by this plugin.
    /// Default returns an empty vec.
    fn skills(&self) -> Vec<SkillDef> {
        Vec::new()
    }

    /// React to a lifecycle event.
    /// Default implementation is a no-op.
    async fn on_event(&self, _event: &PluginEvent) -> Result<(), AppError> {
        Ok(())
    }
}
