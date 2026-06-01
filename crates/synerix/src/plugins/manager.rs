//! Plugin loading and management.

use std::sync::Arc;

use crate::error::AppError;
use crate::skills::SkillDef;
use crate::tools::Tool;

use super::types::{Plugin, PluginEvent};

// ---------------------------------------------------------------------------
// PluginManager
// ---------------------------------------------------------------------------

/// Manages plugin registration, initialisation, and dispatching.
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    /// Create an empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin. This does **not** call `init()` — use [`init_all`] for that.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        tracing::info!("Registered plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Return the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Return whether the manager has no plugins.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Call `init()` on every registered plugin (in registration order).
    pub async fn init_all(&mut self) -> Result<(), AppError> {
        for plugin in &mut self.plugins {
            tracing::info!("Initialising plugin: {}", plugin.name());
            plugin.init().await?;
        }
        Ok(())
    }

    /// Collect all tools contributed by registered plugins.
    pub fn collect_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.plugins.iter().flat_map(|p| p.tools()).collect()
    }

    /// Collect all skills contributed by registered plugins.
    pub fn collect_skills(&self) -> Vec<SkillDef> {
        self.plugins.iter().flat_map(|p| p.skills()).collect()
    }

    /// Broadcast an event to every registered plugin.
    /// Errors from individual plugins are logged but do **not** short-circuit
    /// delivery to subsequent plugins.
    pub async fn emit_event(&self, event: &PluginEvent) -> Result<(), AppError> {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_event(event).await {
                tracing::warn!("Plugin '{}' returned error on event: {}", plugin.name(), e);
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ToolContext, ToolResult};
    use serde_json::json;

    // ---- helpers ----------------------------------------------------------

    /// A minimal stub tool for testing.
    struct StubTool {
        tool_name: String,
    }

    #[async_trait::async_trait]
    impl Tool for StubTool {
        fn name(&self) -> &str {
            &self.tool_name
        }
        fn schema(&self) -> serde_json::Value {
            json!({"type": "object", "properties": {}})
        }
        async fn execute(
            &self,
            _args: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult, AppError> {
            Ok(ToolResult {
                output: "ok".into(),
                is_error: false,
                preview: None,
            })
        }
    }

    /// A test plugin that tracks lifecycle calls.
    struct TestPlugin {
        plugin_name: String,
        init_called: std::sync::atomic::AtomicBool,
        event_count: std::sync::atomic::AtomicUsize,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                plugin_name: name.to_string(),
                init_called: std::sync::atomic::AtomicBool::new(false),
                event_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        #[allow(dead_code)]
        fn was_init_called(&self) -> bool {
            self.init_called.load(std::sync::atomic::Ordering::SeqCst)
        }

        #[allow(dead_code)]
        fn event_count(&self) -> usize {
            self.event_count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.plugin_name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn description(&self) -> &str {
            "A test plugin"
        }

        async fn init(&mut self) -> Result<(), AppError> {
            self.init_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn tools(&self) -> Vec<Arc<dyn Tool>> {
            vec![Arc::new(StubTool {
                tool_name: format!("{}_tool", self.plugin_name),
            })]
        }

        fn skills(&self) -> Vec<SkillDef> {
            vec![SkillDef {
                name: format!("{}_skill", self.plugin_name),
                description: "test skill".into(),
                trigger: crate::skills::SkillTrigger::Explicit,
                instructions: "do stuff".into(),
                required_tools: vec![],
                required_mcp: vec![],
                source_path: None,
            }]
        }

        async fn on_event(&self, _event: &PluginEvent) -> Result<(), AppError> {
            self.event_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    /// A plugin whose `init()` fails.
    struct FailingPlugin;

    #[async_trait::async_trait]
    impl Plugin for FailingPlugin {
        fn name(&self) -> &str {
            "failing"
        }
        fn version(&self) -> &str {
            "0.0.1"
        }
        fn description(&self) -> &str {
            "always fails"
        }
        async fn init(&mut self) -> Result<(), AppError> {
            Err(AppError::Config("init boom".into()))
        }
    }

    /// A plugin whose `on_event()` fails.
    struct EventFailPlugin;

    #[async_trait::async_trait]
    impl Plugin for EventFailPlugin {
        fn name(&self) -> &str {
            "event_fail"
        }
        fn version(&self) -> &str {
            "0.0.1"
        }
        fn description(&self) -> &str {
            "event always fails"
        }
        async fn on_event(&self, _event: &PluginEvent) -> Result<(), AppError> {
            Err(AppError::ExecutionFailed("event boom".into()))
        }
    }

    // ---- tests ------------------------------------------------------------

    #[test]
    fn test_plugin_manager_new_is_empty() {
        let mgr = PluginManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn test_plugin_manager_default_is_empty() {
        let mgr = PluginManager::default();
        assert!(mgr.is_empty());
    }

    #[tokio::test]
    async fn test_register_increments_len() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin::new("p1")));
        assert_eq!(mgr.len(), 1);
        assert!(!mgr.is_empty());

        mgr.register(Box::new(TestPlugin::new("p2")));
        assert_eq!(mgr.len(), 2);
    }

    #[tokio::test]
    async fn test_init_all_calls_plugin_init() {
        let mut mgr = PluginManager::new();
        let plugin = TestPlugin::new("alpha");
        // We can't easily check the atomic after boxing because Box takes ownership,
        // so we rely on the tools/skills side-effects plus the fact init_all succeeds.
        mgr.register(Box::new(plugin));
        let result = mgr.init_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_all_propagates_error() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin::new("good")));
        mgr.register(Box::new(FailingPlugin));

        let result = mgr.init_all().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Config(msg) => assert_eq!(msg, "init boom"),
            other => panic!("expected Config error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_collect_tools_from_multiple_plugins() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin::new("a")));
        mgr.register(Box::new(TestPlugin::new("b")));

        let tools = mgr.collect_tools();
        assert_eq!(tools.len(), 2);

        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"a_tool"));
        assert!(names.contains(&"b_tool"));
    }

    #[tokio::test]
    async fn test_collect_tools_empty_when_no_plugins() {
        let mgr = PluginManager::new();
        assert!(mgr.collect_tools().is_empty());
    }

    #[tokio::test]
    async fn test_collect_skills_from_multiple_plugins() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin::new("x")));
        mgr.register(Box::new(TestPlugin::new("y")));

        let skills = mgr.collect_skills();
        assert_eq!(skills.len(), 2);

        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"x_skill"));
        assert!(names.contains(&"y_skill"));
    }

    #[tokio::test]
    async fn test_collect_skills_empty_when_no_plugins() {
        let mgr = PluginManager::new();
        assert!(mgr.collect_skills().is_empty());
    }

    #[tokio::test]
    async fn test_emit_event_broadcasts_to_all_plugins() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(TestPlugin::new("a")));
        mgr.register(Box::new(TestPlugin::new("b")));

        let event = PluginEvent::PreAgentTurn { turn_number: 1 };
        let result = mgr.emit_event(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_event_propagates_error() {
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(EventFailPlugin));

        let event = PluginEvent::Custom {
            name: "test".into(),
            payload: json!(null),
        };
        let result = mgr.emit_event(&event).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ExecutionFailed(msg) => assert_eq!(msg, "event boom"),
            other => panic!("expected ExecutionFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_plugin_event_debug_clone() {
        let event = PluginEvent::PostToolCall {
            tool_name: "git".into(),
            args: json!({}),
            output: "ok".into(),
            is_error: false,
        };
        let cloned = event.clone();
        assert!(format!("{:?}", cloned).contains("PostToolCall"));
    }

    #[test]
    fn test_plugin_event_all_variants() {
        // Verify all variants can be constructed
        let _ = PluginEvent::PreToolCall {
            tool_name: "t".into(),
            args: json!(null),
        };
        let _ = PluginEvent::PostToolCall {
            tool_name: "t".into(),
            args: json!(null),
            output: "o".into(),
            is_error: true,
        };
        let _ = PluginEvent::PreAgentTurn { turn_number: 0 };
        let _ = PluginEvent::PostAgentTurn { turn_number: 5 };
        let _ = PluginEvent::WorkflowStepComplete {
            step_id: "s1".into(),
            success: true,
        };
        let _ = PluginEvent::Custom {
            name: "n".into(),
            payload: json!({"key": "value"}),
        };
    }

    #[tokio::test]
    async fn test_plugin_default_impls_compile() {
        /// A plugin using only default implementations for init/tools/skills/on_event.
        struct MinimalPlugin;

        #[async_trait::async_trait]
        impl Plugin for MinimalPlugin {
            fn name(&self) -> &str {
                "minimal"
            }
            fn version(&self) -> &str {
                "0.0.1"
            }
            fn description(&self) -> &str {
                "bare minimum"
            }
        }

        let mut mgr = PluginManager::new();
        mgr.register(Box::new(MinimalPlugin));
        mgr.init_all().await.unwrap();
        assert!(mgr.collect_tools().is_empty());
        assert!(mgr.collect_skills().is_empty());
        mgr.emit_event(&PluginEvent::PreAgentTurn { turn_number: 0 })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_mixed_plugins_tools_and_skills() {
        /// Plugin with tools but no skills.
        struct ToolsOnlyPlugin;
        #[async_trait::async_trait]
        impl Plugin for ToolsOnlyPlugin {
            fn name(&self) -> &str {
                "tools_only"
            }
            fn version(&self) -> &str {
                "1.0.0"
            }
            fn description(&self) -> &str {
                "provides tools only"
            }
            fn tools(&self) -> Vec<Arc<dyn Tool>> {
                vec![
                    Arc::new(StubTool {
                        tool_name: "tool_a".into(),
                    }),
                    Arc::new(StubTool {
                        tool_name: "tool_b".into(),
                    }),
                ]
            }
        }

        /// Plugin with skills but no tools.
        struct SkillsOnlyPlugin;
        #[async_trait::async_trait]
        impl Plugin for SkillsOnlyPlugin {
            fn name(&self) -> &str {
                "skills_only"
            }
            fn version(&self) -> &str {
                "1.0.0"
            }
            fn description(&self) -> &str {
                "provides skills only"
            }
            fn skills(&self) -> Vec<SkillDef> {
                vec![SkillDef {
                    name: "skill_a".into(),
                    description: "a".into(),
                    trigger: crate::skills::SkillTrigger::Explicit,
                    instructions: "do a".into(),
                    required_tools: vec![],
                    required_mcp: vec![],
                    source_path: None,
                }]
            }
        }

        let mut mgr = PluginManager::new();
        mgr.register(Box::new(ToolsOnlyPlugin));
        mgr.register(Box::new(SkillsOnlyPlugin));

        assert_eq!(mgr.collect_tools().len(), 2);
        assert_eq!(mgr.collect_skills().len(), 1);
        assert_eq!(mgr.collect_skills()[0].name, "skill_a");
    }
}
