//! LLM unified adapter layer

pub mod adapter;
pub mod provider;
pub mod stream;
pub mod types;

pub use adapter::LlmAdapter;
pub use provider::create_provider;
pub use types::{
    ChatMessage, ChatResponse, ChunkDelta, FunctionSchema, MessageRole, StreamChunk, ToolCall,
    ToolSchema,
};
