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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_plugin_event_pre_tool_call() {
        let event = PluginEvent::PreToolCall {
            tool_name: "read_file".into(),
            args: json!({"path": "test.rs"}),
        };
        assert!(matches!(event, PluginEvent::PreToolCall { .. }));
        if let PluginEvent::PreToolCall { tool_name, args } = event {
            assert_eq!(tool_name, "read_file");
            assert_eq!(args, json!({"path": "test.rs"}));
        }
    }

    #[test]
    fn test_plugin_event_post_tool_call() {
        let event = PluginEvent::PostToolCall {
            tool_name: "shell_exec".into(),
            args: json!({"cmd": "ls"}),
            output: "file1\nfile2".into(),
            is_error: false,
        };
        assert!(matches!(event, PluginEvent::PostToolCall { .. }));
    }

    #[test]
    fn test_plugin_event_post_tool_call_error() {
        let event = PluginEvent::PostToolCall {
            tool_name: "shell_exec".into(),
            args: json!({}),
            output: "permission denied".into(),
            is_error: true,
        };
        if let PluginEvent::PostToolCall { is_error, .. } = event {
            assert!(is_error);
        }
    }

    #[test]
    fn test_plugin_event_agent_turns() {
        let pre = PluginEvent::PreAgentTurn { turn_number: 0 };
        assert!(matches!(pre, PluginEvent::PreAgentTurn { turn_number: 0 }));

        let post = PluginEvent::PostAgentTurn { turn_number: 5 };
        assert!(matches!(
            post,
            PluginEvent::PostAgentTurn { turn_number: 5 }
        ));
    }

    #[test]
    fn test_plugin_event_workflow_step() {
        let success = PluginEvent::WorkflowStepComplete {
            step_id: "build".into(),
            success: true,
        };
        assert!(matches!(
            success,
            PluginEvent::WorkflowStepComplete { success: true, .. }
        ));

        let failed = PluginEvent::WorkflowStepComplete {
            step_id: "test".into(),
            success: false,
        };
        assert!(matches!(
            failed,
            PluginEvent::WorkflowStepComplete { success: false, .. }
        ));
    }

    #[test]
    fn test_plugin_event_custom() {
        let event = PluginEvent::Custom {
            name: "my_hook".into(),
            payload: json!({"key": "value"}),
        };
        if let PluginEvent::Custom { name, payload } = event {
            assert_eq!(name, "my_hook");
            assert_eq!(payload, json!({"key": "value"}));
        }
    }

    #[test]
    fn test_plugin_event_clone() {
        let event = PluginEvent::PreAgentTurn { turn_number: 1 };
        let cloned = event.clone();
        assert!(matches!(
            cloned,
            PluginEvent::PreAgentTurn { turn_number: 1 }
        ));
    }

    #[test]
    fn test_plugin_event_debug() {
        let event = PluginEvent::PreAgentTurn { turn_number: 0 };
        let debug = format!("{:?}", event);
        assert!(debug.contains("PreAgentTurn"));
    }
}
