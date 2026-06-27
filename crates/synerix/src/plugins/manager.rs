//! PluginManager — 插件注册与生命周期管理
//!
use std::sync::Arc;

use crate::error::AppError;
use crate::skills::SkillDef;
use crate::tools::Tool;

use super::types::{Plugin, PluginEvent};

/// 插件管理器，负责注册、初始化、工具/技能收集和事件广播
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    /// 创建空的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 注册一个插件（不调用 init，需后续主动调用 init_all）
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        tracing::info!("Registered plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// 已注册插件数量
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// 是否无插件
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// 调用所有插件的 init() 方法
    ///
    /// # 错误处理
    /// 单个插件失败不会阻断其他插件初始化。失败信息聚合后
    /// 以 PluginInitPartialFailure 错误返回。
    ///
    pub async fn init_all(&mut self) -> Result<(), AppError> {
        let total_count = self.plugins.len();
        let mut results = Vec::with_capacity(total_count);
        for plugin in &mut self.plugins {
            let name = plugin.name().to_string();
            tracing::info!("Initialising plugin: {name}");
            results.push(plugin.init().await);
        }
        aggregate_init_results(results, total_count)
    }

    /// 收集所有插件贡献的工具
    pub fn collect_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.plugins.iter().flat_map(|p| p.tools()).collect()
    }

    /// 收集所有插件贡献的技能
    pub fn collect_skills(&self) -> Vec<SkillDef> {
        self.plugins.iter().flat_map(|p| p.skills()).collect()
    }

    /// 向所有已注册插件广播事件
    ///
    /// # 错误处理
    /// 单个插件处理失败不会阻断后续插件。错误汇聚后
    /// 以 PluginEventPartialFailure 返回。
    pub async fn emit_event(&self, event: &PluginEvent) -> Result<(), AppError> {
        let mut errors: Vec<(String, AppError)> = Vec::new();
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_event(event).await {
                tracing::warn!("Plugin '{}' returned error on event: {}", plugin.name(), e);
                errors.push((plugin.name().to_string(), e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::PluginEventPartialFailure {
                failed_count: errors.len(),
                total_count: self.plugins.len(),
            })
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 聚合 init 结果：统计失败数，无失败返回 Ok，否则返回 PartialFailure
fn aggregate_init_results(
    results: Vec<Result<(), AppError>>,
    total_count: usize,
) -> Result<(), AppError> {
    let failed_count = results.iter().filter(|r| r.is_err()).count();
    if failed_count == 0 {
        Ok(())
    } else {
        Err(AppError::PluginInitPartialFailure {
            failed_count,
            total_count,
        })
    }
}

// ── 测试套件 ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::stubs::{
        EventFailPlugin, FailingPlugin, MinimalPlugin, SkillsOnlyPlugin, TestPlugin,
        ToolsOnlyPlugin,
    };
    use crate::plugins::PluginEvent;
    use serde_json::json;

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
            AppError::PluginInitPartialFailure {
                failed_count,
                total_count,
            } => {
                assert_eq!(failed_count, 1);
                assert_eq!(total_count, 2);
            }
            other => panic!("expected PluginInitPartialFailure, got {:?}", other),
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
        mgr.register(Box::new(TestPlugin::new("b")));

        let event = PluginEvent::Custom {
            name: "test".into(),
            payload: json!(null),
        };
        let result = mgr.emit_event(&event).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::PluginEventPartialFailure {
                failed_count,
                total_count,
            } => {
                assert_eq!(failed_count, 1);
                assert_eq!(total_count, 2);
            }
            other => panic!("expected PluginEventPartialFailure, got {:?}", other),
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
        let mut mgr = PluginManager::new();
        mgr.register(Box::new(ToolsOnlyPlugin));
        mgr.register(Box::new(SkillsOnlyPlugin));

        assert_eq!(mgr.collect_tools().len(), 2);
        assert_eq!(mgr.collect_skills().len(), 1);
        assert_eq!(mgr.collect_skills()[0].name, "skill_a");
    }
}
