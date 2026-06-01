//! Core Agentic Loop — stream → tool dispatch → continue
//!
//! Optimized with:
//! - HashMap for O(1) tool call accumulation
//! - Parallel tool execution via `futures::future::join_all`
//! - Streaming token counter
//! - Per-tool execution timeout

use std::collections::HashMap;

use futures::StreamExt;
use tokio::sync::mpsc;

use crate::agent::context::ContextManager;
use crate::agent::tool_dispatcher::{dispatch_with_timeout, PendingToolCall};
use crate::app::AgentEvent;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::{ChatMessage, ChunkDelta, MessageRole, ToolCall};
use crate::mcp::manager::McpManager;
use crate::token_estimator::estimate_tokens;
use crate::tools::registry::ToolRegistry;

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
                    let result = dispatch_with_timeout(&tc_name, &args, tools, mcp).await;
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
