//! MCP (Model Context Protocol) client

pub mod manager;
pub mod mcp_client;
pub mod transport;
pub mod types;

pub use manager::McpManager;
pub use mcp_client::McpClient;
pub use types::{JsonRpcRequest, JsonRpcResponse, McpToolDef, McpToolResult};
