//! Core Agentic Loop — stream → tool dispatch → continue
//!
//! Optimized with:
//! - HashMap for O(1) tool call accumulation
//! - Parallel tool execution via `futures::future::join_all`
//! - Streaming token counter
//! - Per-tool execution timeout

use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::mpsc;

use crate::agent::context::ContextManager;
use crate::app::AgentEvent;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::{ChatMessage, ChunkDelta, MessageRole, ToolCall};
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;
use crate::tools::trait_def::ToolContext;

/// Tool execution timeout in seconds
const TOOL_TIMEOUT_SECS: u64 = 120;

/// Pending tool call accumulator (for streaming JSON args)
struct PendingToolCall {
    id: String,
    name: String,
    args_buffer: String,
}

/// Run the agent reasoning loop
///
/// Flow: User input → LLM → (text | tool_call)* → Done
/// Each tool_call: dispatch → result → back to LLM
pub async fn run_agent_loop(
    llm: &dyn LlmAdapter,
    ctx: &mut ContextManager,
    tools: &ToolRegistry,
    mcp: &McpManager,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    max_turns: usize,
) -> Result<(), AppError> {
    for turn in 0..max_turns {
        tracing::debug!("Agent turn {}/{}", turn + 1, max_turns);

        // Collect schemas from tools + MCP
        let mut schemas = tools.all_schemas();
        schemas.extend(mcp.tool_schemas());

        // Stream from LLM
        let stream = llm.chat_stream(ctx.messages(), &schemas).await?;
        let mut stream = std::pin::pin!(stream);

        let mut full_text = String::new();
        let mut tool_calls: HashMap<String, PendingToolCall> = HashMap::new();
        let mut total_tokens = 0usize;

        // Process stream chunks
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;

            match chunk.delta {
                ChunkDelta::Text { content } => {
                    total_tokens += estimate_tokens(&content);
                    full_text.push_str(&content);
                    event_tx.send(AgentEvent::TextDelta(content))?;
                }
                ChunkDelta::ToolCall {
                    id,
                    name,
                    args_delta,
                } => {
                    // O(1) lookup via HashMap
                    if let Some(existing) = tool_calls.get_mut(&id) {
                        existing.args_buffer.push_str(&args_delta);
                    } else {
                        tool_calls.insert(
                            id.clone(),
                            PendingToolCall {
                                id,
                                name,
                                args_buffer: args_delta,
                            },
                        );
                    }
                }
                ChunkDelta::Done => break,
            }
        }

        tracing::debug!("Turn {} streamed ~{} tokens", turn + 1, total_tokens);

        // If no tool calls, reasoning is complete
        if tool_calls.is_empty() {
            if !full_text.is_empty() {
                ctx.push(ChatMessage::assistant(&full_text));
            }
            event_tx.send(AgentEvent::Done)?;
            return Ok(());
        }

        // Add assistant message with tool calls
        let resolved_calls: Vec<ToolCall> = tool_calls
            .values()
            .map(|tc| ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.args_buffer.clone(),
            })
            .collect();

        ctx.push(ChatMessage {
            role: MessageRole::Assistant,
            content: if full_text.is_empty() {
                None
            } else {
                Some(full_text.clone())
            },
            tool_calls: Some(resolved_calls),
            tool_call_id: None,
            name: None,
        });

        // Execute tool calls in parallel
        let tool_futures: Vec<_> = tool_calls
            .values()
            .map(|tc| {
                let args: serde_json::Value = if tc.args_buffer.is_empty() {
                    serde_json::json!({})
                } else {
                    serde_json::from_str(&tc.args_buffer).unwrap_or_else(|_| serde_json::json!({}))
                };

                // Emit ToolCallStart event
                let _ = event_tx.send(AgentEvent::ToolCallStart {
                    name: tc.name.clone(),
                    args: args.clone(),
                });

                let tc_id = tc.id.clone();
                let tc_name = tc.name.clone();

                async move {
                    let result = dispatch_tool_with_timeout(&tc_name, &args, tools, mcp).await;
                    let (output, is_error) = match result {
                        Ok(r) => (r.output, r.is_error),
                        Err(e) => (format!("Error: {}", e), true),
                    };
                    (tc_id, tc_name, output, is_error)
                }
            })
            .collect();

        // Await all tool calls concurrently
        let results = futures::future::join_all(tool_futures).await;

        // Process results
        for (tc_id, tc_name, output, is_error) in results {
            let _ = event_tx.send(AgentEvent::ToolResult {
                name: tc_name,
                output: output.clone(),
                is_error,
            });

            // Add tool result to context
            ctx.push(ChatMessage::tool_result(tc_id, output));
        }

        // Loop continues → LLM reasons over tool results
    }

    Err(AppError::MaxTurnsExceeded(max_turns))
}

/// Dispatch a tool call with timeout: local tools first, then MCP
async fn dispatch_tool_with_timeout(
    name: &str,
    args: &serde_json::Value,
    tools: &ToolRegistry,
    mcp: &McpManager,
) -> Result<crate::tools::trait_def::ToolResult, AppError> {
    let timeout = Duration::from_secs(TOOL_TIMEOUT_SECS);

    tokio::time::timeout(timeout, dispatch_tool(name, args, tools, mcp))
        .await
        .map_err(|_| {
            AppError::ExecutionFailed(format!(
                "Tool '{}' timed out after {}s",
                name, TOOL_TIMEOUT_SECS
            ))
        })?
}

/// Dispatch a tool call: local tools first, then MCP
async fn dispatch_tool(
    name: &str,
    args: &serde_json::Value,
    tools: &ToolRegistry,
    mcp: &McpManager,
) -> Result<crate::tools::trait_def::ToolResult, AppError> {
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
        return Ok(crate::tools::trait_def::ToolResult {
            output: result.content,
            is_error: result.is_error,
            preview: None,
        });
    }

    Err(AppError::ToolNotFound(name.to_string()))
}

/// Rough token estimation (~4 chars per token for English, ~2 for CJK)
fn estimate_tokens(text: &str) -> usize {
    let cjk_count = text
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp) || // CJK Unified
            (0x3400..=0x4DBF).contains(&cp) || // CJK Extension A
            (0xF900..=0xFAFF).contains(&cp) // CJK Compatibility
        })
        .count();

    let other_count = text.len() - cjk_count;
    (other_count / 4) + (cjk_count / 2) + 1
}
