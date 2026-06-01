//! MCP (Model Context Protocol) client

pub mod mcp_client;
pub mod manager;
pub mod transport;
pub mod types;

pub use mcp_client::McpClient;
pub use manager::McpManager;
pub use types::{JsonRpcRequest, JsonRpcResponse, McpToolDef, McpToolResult};
