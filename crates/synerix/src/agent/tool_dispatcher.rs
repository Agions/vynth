//! Tool dispatch with timeout, MCP fallback, and approval flow

use std::time::Duration;

use crate::error::AppError;
use crate::mcp::manager::McpManager;
use crate::sandbox::approval::{ApprovalHandler, AutoApprove};
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
    approval_handler: Option<&dyn ApprovalHandler>,
) -> Result<ToolResult, AppError> {
    let timeout = Duration::from_secs(timeout_secs);

    tokio::time::timeout(timeout, dispatch(name, args, tools, mcp, approval_handler))
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
    approval_handler: Option<&dyn ApprovalHandler>,
) -> Result<ToolResult, AppError> {
    // Try local tool first
    if let Some(tool) = tools.get(name) {
        let ctx = ToolContext::default();

        // Check if approval needed
        if tool.requires_approval(args) {
            let preview = format!("{} {}", name, serde_json::to_string_pretty(args).unwrap_or_default());
            let handler = approval_handler.unwrap_or(&AutoApprove);
            let decision = handler.request_approval(&preview).await?;
            match decision {
                crate::sandbox::approval::ApprovalDecision::Allow
                | crate::sandbox::approval::ApprovalDecision::AllowAlways => {}
                crate::sandbox::approval::ApprovalDecision::Deny => {
                    return Ok(ToolResult {
                        output: format!("❌ Tool '{}' execution denied by user", name),
                        is_error: true,
                        preview: None,
                    });
                }
            }
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
