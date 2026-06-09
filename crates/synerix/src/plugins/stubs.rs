//! 插件系统测试桩 — 供各测试模块复用
//!
//! # 优化说明
//! 将原 manager.rs 中的 5 个测试插件 + 1 个测试工具的
//! 实现提取为独立模块，避免测试代码与生产代码混合。

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use crate::error::AppError;
use crate::plugins::{Plugin, PluginEvent};
use crate::skills::SkillDef;
use crate::tools::{Tool, ToolContext, ToolResult};
use serde_json::json;

// ── 桩：测试工具 ─────────────────────────────────────────────────────────────────

/// 最小化测试桩工具
pub struct StubTool {
    pub tool_name: String,
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

// ── 桩：通用测试插件 ─────────────────────────────────────────────────────────────

/// 通用测试插件，追踪生命周期调用
pub struct TestPlugin {
    pub plugin_name: String,
    pub init_called: AtomicBool,
    pub event_count: AtomicUsize,
}

impl TestPlugin {
    pub fn new(name: &str) -> Self {
        Self {
            plugin_name: name.to_string(),
            init_called: AtomicBool::new(false),
            event_count: AtomicUsize::new(0),
        }
    }

    pub fn was_init_called(&self) -> bool {
        self.init_called.load(Ordering::SeqCst)
    }

    pub fn event_count(&self) -> usize {
        self.event_count.load(Ordering::SeqCst)
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
        self.init_called.store(true, Ordering::SeqCst);
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
        self.event_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

// ── 桩：初始化失败的插件 ─────────────────────────────────────────────────────────

pub struct FailingPlugin;

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

// ── 桩：事件处理失败的插件 ───────────────────────────────────────────────────────

pub struct EventFailPlugin;

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

// ── 桩：最小化插件（使用所有默认实现） ─────────────────────────────────────────

pub struct MinimalPlugin;

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

// ── 桩：仅提供工具的插件 ───────────────────────────────────────────────────────

pub struct ToolsOnlyPlugin;

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

// ── 桩：仅提供技能的插件 ───────────────────────────────────────────────────────

pub struct SkillsOnlyPlugin;

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
