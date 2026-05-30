//! LLM unified adapter layer

pub mod adapter;
pub mod provider;
pub mod types;
pub mod stream;

pub use adapter::LlmAdapter;
pub use types::{ChatMessage, ChatResponse, ChunkDelta, MessageRole, StreamChunk, ToolCall, ToolSchema, FunctionSchema};
pub use provider::create_provider;
