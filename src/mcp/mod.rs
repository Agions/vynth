//! MCP (Model Context Protocol) client

pub mod client;
pub mod manager;
pub mod transport;
pub mod types;

pub use client::McpClient;
pub use manager::McpManager;
pub use types::{JsonRpcRequest, JsonRpcResponse, McpToolDef, McpToolResult};
