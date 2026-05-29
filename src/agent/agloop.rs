//! Core Agentic Loop — stream → tool dispatch → continue

use futures::StreamExt;
use tokio::sync::mpsc;

use crate::app::AgentEvent;
use crate::error::AppError;
use crate::agent::context::ContextManager;
use crate::llm::adapter::LlmAdapter;
use crate::llm::types::{ChatMessage, ChunkDelta, MessageRole, ToolCall};
use crate::mcp::manager::McpManager;
use crate::tools::registry::ToolRegistry;
use crate::tools::trait_def::ToolContext;

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
        let mut tool_calls: Vec<PendingToolCall> = Vec::new();

        // Process stream chunks
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;

            match chunk.delta {
                ChunkDelta::Text { content } => {
                    full_text.push_str(&content);
                    event_tx.send(AgentEvent::TextDelta(content))?;
                }
                ChunkDelta::ToolCall {
                    id,
                    name,
                    args_delta,
                } => {
                    // Accumulate tool call args (streaming JSON)
                    if let Some(existing) = tool_calls.iter_mut().find(|tc| tc.id == id) {
                        existing.args_buffer.push_str(&args_delta);
                    } else {
                        tool_calls.push(PendingToolCall {
                            id,
                            name,
                            args_buffer: args_delta,
                        });
                    }
                }
                ChunkDelta::Done => break,
            }
        }

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
            .iter()
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

        // Execute each tool call
        for tc in &tool_calls {
            let args: serde_json::Value = if tc.args_buffer.is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&tc.args_buffer)
                    .unwrap_or_else(|_| serde_json::json!({}))
            };

            event_tx.send(AgentEvent::ToolCallStart {
                name: tc.name.clone(),
                args: args.clone(),
            })?;

            let result = dispatch_tool(tc, &args, tools, mcp).await;

            let (output, is_error) = match result {
                Ok(r) => (r.output, r.is_error),
                Err(e) => (format!("Error: {}", e), true),
            };

            event_tx.send(AgentEvent::ToolResult {
                name: tc.name.clone(),
                output: output.clone(),
                is_error,
            })?;

            // Add tool result to context
            ctx.push(ChatMessage::tool_result(tc.id.clone(), output));
        }

        // Loop continues → LLM reasons over tool results
    }

    Err(AppError::MaxTurnsExceeded(max_turns))
}

/// Dispatch a tool call: local tools first, then MCP
async fn dispatch_tool(
    tc: &PendingToolCall,
    args: &serde_json::Value,
    tools: &ToolRegistry,
    mcp: &McpManager,
) -> Result<crate::tools::trait_def::ToolResult, AppError> {
    // Try local tool first
    if let Some(tool) = tools.get(&tc.name) {
        let ctx = ToolContext::default();

        // Check if approval needed
        if tool.requires_approval(args) {
            // TODO: Integrate with TUI approval flow
            tracing::info!("Tool '{}' requires approval (auto-approving for now)", tc.name);
        }

        return tool.execute(args.clone(), &ctx).await;
    }

    // Try MCP tool (format: mcp__server__tool)
    if let Some((server, tool_name)) = mcp.find_tool(&tc.name) {
        let result = mcp.call_tool(server, tool_name, args.clone()).await?;
        return Ok(crate::tools::trait_def::ToolResult {
            output: result.content,
            is_error: result.is_error,
            preview: None,
        });
    }

    Err(AppError::ToolNotFound(tc.name.clone()))
}
