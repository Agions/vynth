//! Tool dispatch with timeout and MCP fallback

use std::time::Duration;

use crate::error::AppError;
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;
use crate::tools::traits::{ToolContext, ToolResult};

/// Pending tool call accumulator (for streaming JSON args)
pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub args_buffer: String,
}

/// Dispatch a tool call with timeout: local tools first, then MCP
pub async fn dispatch_with_timeout(
    name: &str,
    args: &serde_json::Value,
    tools: &ToolRegistry,
    mcp: &McpManager,
    timeout_secs: u64,
) -> Result<ToolResult, AppError> {
    let timeout = Duration::from_secs(timeout_secs);

    tokio::time::timeout(timeout, dispatch(name, args, tools, mcp))
        .await
        .map_err(|_| {
            AppError::ExecutionFailed(format!("Tool '{}' timed out after {}s", name, timeout_secs))
        })?
}

/// Dispatch a tool call: local tools first, then MCP fallback
async fn dispatch(
    name: &str,
    args: &serde_json::Value,
    tools: &ToolRegistry,
    mcp: &McpManager,
) -> Result<ToolResult, AppError> {
    // Try local tool first
    if let Some(tool) = tools.get(name) {
        let ctx = ToolContext::default();

        // Check if approval needed
        if tool.requires_approval(args) {
            // TODO: Integrate with TUI approval flow
            tracing::info!("Tool '{}' requires approval (auto-approving for now)", name);
        }

        return tool.execute(args.clone(), &ctx).await;
    }

    // Try MCP tool (format: mcp__server__tool)
    if let Some((server, tool_name)) = mcp.find_tool(name) {
        let result = mcp.call_tool(server, tool_name, args.clone()).await?;
        return Ok(ToolResult {
            output: result.content,
            is_error: result.is_error,
            preview: None,
        });
    }

    Err(AppError::ToolNotFound(name.to_string()))
}
