//! End-to-end test for the agent loop.
//!
//! Creates a full stack (mock LLM, context, tools, MCP, event channel)
//! and verifies the streaming + tool dispatch + event emission cycle.

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{self, Stream};
use tokio::sync::mpsc;

use synerix::agent::{run_agent_loop, ContextManager, TokenBudget};
use synerix::app::events::AgentEvent;
use synerix::error::AppError;
use synerix::llm::adapter::LlmAdapter;
use synerix::llm::types::{ChatMessage, ChatResponse, ChunkDelta, StreamChunk, ToolSchema};
use synerix::mcp::manager::McpManager;
use synerix::tools::builtin::FileReadTool;
use synerix::tools::registry::ToolRegistry;

// ---------------------------------------------------------------------------
// Mock LLM adapter
//
// First call:  yields text + tool-call ("read_file") + Done.
// Subsequent calls: yield text only + Done (so the agent loop terminates).
// ---------------------------------------------------------------------------

struct MockLlmAdapter {
    call_count: AtomicUsize,
}

impl MockLlmAdapter {
    fn new() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
        }
    }

    /// Stream that includes a tool call (for the first invocation).
    fn stream_with_tool_call() -> Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>
    {
        let chunks = vec![
            Ok(StreamChunk {
                delta: ChunkDelta::Text {
                    content: "Hello".to_string(),
                },
            }),
            Ok(StreamChunk {
                delta: ChunkDelta::Text {
                    content: " world!".to_string(),
                },
            }),
            Ok(StreamChunk {
                delta: ChunkDelta::ToolCall {
                    id: "call_1".to_string(),
                    name: "read_file".to_string(),
                    args_delta: "{\"path\":\"/tmp/test\"}".to_string(),
                },
            }),
            Ok(StreamChunk {
                delta: ChunkDelta::Done,
            }),
        ];
        Box::pin(stream::iter(chunks))
    }

    /// Stream that only has text (no tool call) – allows the loop to finish.
    fn stream_text_only() -> Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>> {
        let chunks = vec![
            Ok(StreamChunk {
                delta: ChunkDelta::Text {
                    content: "Task complete.".to_string(),
                },
            }),
            Ok(StreamChunk {
                delta: ChunkDelta::Done,
            }),
        ];
        Box::pin(stream::iter(chunks))
    }
}

#[async_trait]
impl LlmAdapter for MockLlmAdapter {
    /// Non-streaming call – not used by the agent loop; return an error.
    async fn chat(
        &self,
        _messages: &[ChatMessage],
        _tools: &[ToolSchema],
    ) -> Result<ChatResponse, AppError> {
        Err(AppError::Llm("mock_chat not used in agent loop".into()))
    }

    /// Streaming call – alternates behaviour based on call count.
    async fn chat_stream(
        &self,
        _messages: &[ChatMessage],
        _tools: &[ToolSchema],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>, AppError> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if count == 0 {
            Ok(Self::stream_with_tool_call())
        } else {
            Ok(Self::stream_text_only())
        }
    }

    fn model_id(&self) -> &str {
        "mock-model"
    }

    fn context_window(&self) -> usize {
        4096
    }
}

// ---------------------------------------------------------------------------
// Agent loop end-to-end test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_agent_loop_e2e() {
    // 1. Create context with a user message
    let budget = TokenBudget::new(100_000);
    let mut ctx = ContextManager::new(budget);
    ctx.push(ChatMessage::user("Hello, please read /tmp/test"));

    // 2. Register at least one built-in tool (file_read)
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(FileReadTool));

    // 3. Create an empty MCP manager
    let mcp = McpManager::connect_all(&[]).await.unwrap();

    // 4. Event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentEvent>();

    // 5. Run the agent loop in a separate task so we can collect events concurrently.
    //    Return (result, ctx) so we can inspect the final context.
    let llm = MockLlmAdapter::new();
    let loop_handle = tokio::spawn(async move {
        let result = run_agent_loop(&llm, &mut ctx, &tools, &mcp, event_tx, 3, 30).await;
        (result, ctx)
    });

    // 6. Collect events from the channel until Done is received.
    let mut stream_chunks: Vec<String> = Vec::new();
    let mut tool_results: Vec<(String, String, bool)> = Vec::new();
    let mut done_received = false;

    while let Some(event) = event_rx.recv().await {
        match event {
            AgentEvent::TextDelta(text) => {
                stream_chunks.push(text);
            }
            AgentEvent::ToolCallStart { name, .. } => {
                // Record that a tool call started
                tracing::debug!("Tool call started: {}", name);
            }
            AgentEvent::ToolResult {
                name,
                output,
                is_error,
            } => {
                tool_results.push((name, output, is_error));
            }
            AgentEvent::Done => {
                done_received = true;
                // Drain any remaining events before closing
                while let Ok(remaining) = event_rx.try_recv() {
                    if let AgentEvent::TextDelta(t) = remaining {
                        stream_chunks.push(t);
                    }
                }
                break;
            }
            AgentEvent::Error(msg) => {
                panic!("Unexpected agent error event: {}", msg);
            }
        }
    }

    // 7. Verify events ------------------------------------------------------

    assert!(done_received, "Agent loop should emit a Done event");

    // We expect at least one TextDelta event
    assert!(
        !stream_chunks.is_empty(),
        "Should have received at least one TextDelta event"
    );
    let full_text: String = stream_chunks.concat();
    assert!(
        full_text.contains("Hello"),
        "Stream should contain 'Hello', got: {}",
        full_text
    );
    assert!(
        full_text.contains("world!"),
        "Stream should contain 'world!', got: {}",
        full_text
    );

    // The mock produces a ToolCall for "read_file", so the agent loop
    // should dispatch it and emit a ToolResult event.
    assert!(
        !tool_results.is_empty(),
        "Should have received at least one ToolResult event"
    );
    let (tool_name, _output, _is_error) = &tool_results[0];
    assert_eq!(
        tool_name, "read_file",
        "Tool result should be for 'read_file'"
    );

    // 8. Verify loop completed successfully
    let (result, ctx) = loop_handle.await.unwrap();
    assert!(
        result.is_ok(),
        "run_agent_loop should return Ok, got: {:?}",
        result
    );

    // 9. Verify context contains user and assistant messages
    let msgs = ctx.messages();
    assert!(
        msgs.len() >= 2,
        "Context should have at least 2 messages (user + assistant), got {}",
        msgs.len()
    );

    let has_user = msgs.iter().any(|m| m.role.as_str() == "user");
    let has_assistant = msgs.iter().any(|m| m.role.as_str() == "assistant");

    assert!(has_user, "Context should contain a user message");
    assert!(has_assistant, "Context should contain an assistant message");

    // Also verify there's a tool result message (from the read_file dispatch)
    let has_tool = msgs.iter().any(|m| m.role.as_str() == "tool");
    assert!(has_tool, "Context should contain a tool result message");
}
