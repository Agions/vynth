//! MCP (Model Context Protocol) client

pub mod client;
pub mod manager;
pub mod types;
pub mod transport;

pub use manager::McpManager;
pub use client::McpClient;
pub use types::{JsonRpcRequest, JsonRpcResponse, McpToolDef, McpToolResult};
