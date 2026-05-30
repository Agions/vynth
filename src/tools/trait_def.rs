//! Tool trait definition

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::SandboxMode;
use crate::error::AppError;

/// Tool execution result
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
    /// Preview info for user (e.g., diff preview)
    pub preview: Option<String>,
}

/// Tool pluggable trait
/// All tools (built-in + MCP bridge) implement this interface
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (LLM function name)
    fn name(&self) -> &str;

    /// JSON Schema description (sent to LLM)
    fn schema(&self) -> Value;

    /// Whether this tool requires user approval (high-risk: shell, file_write)
    fn requires_approval(&self, args: &Value) -> bool {
        let _ = args;
        false
    }

    /// Execute the tool
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError>;
}

/// Tool execution context
pub struct ToolContext {
    pub working_dir: PathBuf,
    pub sandbox_mode: SandboxMode,
    pub approval_handler: Option<Arc<dyn ApprovalHandler>>,
}

/// Cached working directory (computed once, reused forever)
static CACHED_CWD: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn get_cached_cwd() -> PathBuf {
    CACHED_CWD
        .get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .clone()
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            working_dir: get_cached_cwd(),
            sandbox_mode: SandboxMode::Confirm,
            approval_handler: None,
        }
    }
}

/// Approval handler trait (for sandbox confirmation)
#[async_trait]
pub trait ApprovalHandler: Send + Sync {
    /// Request approval for a tool execution
    async fn request_approval(&self, preview: &str) -> Result<bool, AppError>;
}
