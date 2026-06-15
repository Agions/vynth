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
use crate::coding_modes::CodingMode;
use crate::error::AppError;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::{ChatMessage, ChunkDelta, MessageRole, ToolCall, ToolSchema};
use crate::mcp::manager::McpManager;
use crate::token_estimator::estimate_tokens;
use crate::tools::registry::ToolRegistry;

/// Run the agent reasoning loop
///
/// Flow: User input → LLM → (text | tool_call)* → Done
/// Each tool_call: dispatch → result → back to LLM
///
/// ## Optimizations
/// - Tool schemas collected once before the loop (unchanged between turns)
/// - HashMap reused across turns via `.clear()` instead of reallocation
/// - Tool call arguments pre-parsed to `serde_json::Value` to avoid repeated parsing
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_loop(
    llm: &dyn LlmAdapter,
    ctx: &mut ContextManager,
    tools: &ToolRegistry,
    mcp: &McpManager,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    max_turns: usize,
    tool_timeout_secs: u64,
    coding_mode: CodingMode,
) -> Result<(), AppError> {
    // Cache tool schemas once — they don't change across turns
    let schemas = collect_schemas(tools, mcp);

    // Reusable buffers to avoid re-allocation per turn
    let mut tool_calls: HashMap<String, PendingToolCall> = HashMap::with_capacity(8);
    let mut filtered_calls: Vec<ParsedToolCall> = Vec::with_capacity(8);

    for turn in 0..max_turns {
        tracing::debug!("Agent turn {}/{}", turn + 1, max_turns);

        // Stream from LLM
        let stream = llm.chat_stream(ctx.messages(), &schemas).await?;
        let mut stream = std::pin::pin!(stream);

        let mut full_text = String::new();
        let mut total_tokens = 0usize;

        // Clear reusable buffers (retain capacity)
        tool_calls.clear();
        filtered_calls.clear();

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

        let had_tool_calls = !tool_calls.is_empty();

        // Resolve tool calls — drain() clears + yields ownership in one operation
        let resolved_calls: Vec<ToolCall> = tool_calls
            .drain()
            .map(|(_, tc)| ToolCall {
                id: tc.id,
                name: tc.name,
                arguments: tc.args_buffer,
            })
            .collect();

        // Filter and pre-parse tool calls based on coding mode
        for tc in resolved_calls {
            if !coding_mode.allow_file_write() && tc.name.starts_with("write_file") {
                continue;
            }
            if !coding_mode.allow_command_exec() && tc.name.starts_with("execute_command") {
                continue;
            }
            // Pre-parse arguments once to avoid repeated from_str calls
            let args: serde_json::Value = if tc.arguments.is_empty() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&tc.arguments)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
            };
            filtered_calls.push(ParsedToolCall {
                id: tc.id,
                name: tc.name,
                args,
            });
        }

        if filtered_calls.is_empty() && had_tool_calls {
            event_tx.send(AgentEvent::TextDelta(
                "⚠️ 当前编码模式不允许执行此操作。请切换到 Act 或 Plan 模式。".to_string(),
            ))?;
            return Ok(());
        }

        // Execute tool calls in parallel
        let tool_futures: Vec<_> = filtered_calls
            .iter()
            .map(|tc| {
                // Emit ToolCallStart event — send args reference, clone only once for event
                let _ = event_tx.send(AgentEvent::ToolCallStart {
                    name: tc.name.clone(),
                    args: tc.args.clone(),
                });

                let tc_id = tc.id.clone();
                let tc_name = tc.name.clone();
                // Reference to args inside filtered_calls
                let args_ref = &tc.args;

                async move {
                    let result =
                        dispatch_with_timeout(&tc_name, args_ref, tools, mcp, tool_timeout_secs)
                            .await;
                    let (output, is_error) = match result {
                        Ok(r) => (r.output, r.is_error),
                        Err(e) => (format!("Error: {e}"), true),
                    };
                    (tc_id, tc_name, output, is_error)
                }
            })
            .collect();

        // Await all tool calls concurrently
        let results = futures::future::join_all(tool_futures).await;

        // Add assistant message with tool calls — move full_text instead of clone
        ctx.push(ChatMessage {
            role: MessageRole::Assistant,
            content: if full_text.is_empty() {
                None
            } else {
                Some(full_text)
            },
            tool_calls: Some(
                filtered_calls
                    .iter()
                    .map(|tc| ToolCall {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: String::new(), // args already dispatched, don't need serialized copy
                    })
                    .collect(),
            ),
            tool_call_id: None,
            name: None,
        });

        // Process results — send event first, then move output
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

/// A tool call with pre-parsed JSON arguments — avoids repeated `serde_json::from_str`.
struct ParsedToolCall {
    id: String,
    name: String,
    args: serde_json::Value,
}

/// Collect tool schemas from the tool registry and MCP manager once.
/// The result is immutable across all turns of a single agent loop.
fn collect_schemas(tools: &ToolRegistry, mcp: &McpManager) -> Vec<ToolSchema> {
    let mut schemas = tools.all_schemas();
    schemas.extend(mcp.tool_schemas());
    schemas
}
