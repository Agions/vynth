//! MCP Client — single server connection

use crate::config::McpServerConfig;
use crate::error::AppError;
use crate::mcp::transport::{McpTransport, StdioTransport};
use crate::mcp::types::*;

/// Single MCP Server connection
pub struct McpClient {
    config: McpServerConfig,
    transport: Option<Box<dyn McpTransport>>,
    tools: Vec<McpToolDef>,
}

impl McpClient {
    /// Connect to an MCP server
    pub async fn connect(config: McpServerConfig) -> Result<Self, AppError> {
        let transport: Box<dyn McpTransport> = match &config.transport {
            crate::config::McpTransport::Stdio { command, args } => {
                Box::new(StdioTransport::connect(command, args).await?)
            }
            crate::config::McpTransport::Http { url } => {
                return Err(AppError::McpTransport(format!(
                    "HTTP transport not yet implemented: {}",
                    url
                )));
            }
        };

        let mut client = Self {
            config,
            transport: Some(transport),
            tools: Vec::new(),
        };

        // Discover tools
        client.discover_tools().await?;

        Ok(client)
    }

    /// Discover available tools from the server
    async fn discover_tools(&mut self) -> Result<(), AppError> {
        let request = JsonRpcRequest::new(1, "tools/list", None);

        if let Some(transport) = &self.transport {
            let response = transport.send_and_wait(request).await?;

            if let Some(result) = response.result {
                if let Some(tools) = result.get("tools").and_then(|v| v.as_array()) {
                    self.tools = tools
                        .iter()
                        .filter_map(|t| serde_json::from_value(t.clone()).ok())
                        .collect();
                }
            }
        }

        tracing::info!(
            "MCP server '{}': discovered {} tools",
            self.config.name,
            self.tools.len()
        );

        Ok(())
    }

    /// Call a tool on this server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<McpToolResult, AppError> {
        // Permission check
        if !self.is_tool_allowed(tool_name) {
            return Err(AppError::McpPermissionDenied {
                server: self.config.name.clone(),
                tool: tool_name.to_string(),
            });
        }

        let request = JsonRpcRequest::new(
            2,
            "tools/call",
            Some(serde_json::json!({
                "name": tool_name,
                "arguments": args
            })),
        );

        if let Some(transport) = &self.transport {
            let response = transport.send_and_wait(request).await?;

            if let Some(error) = response.error {
                return Ok(McpToolResult {
                    content: format!("MCP error: {}", error.message),
                    is_error: true,
                });
            }

            if let Some(result) = response.result {
                let content = result["content"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|c| c["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_else(|| serde_json::to_string_pretty(&result).unwrap_or_default());

                let is_error = result
                    .get("isError")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                return Ok(McpToolResult { content, is_error });
            }
        }

        Err(AppError::McpTransport("Not connected".to_string()))
    }

    /// Check if a tool name is allowed by this server's permission config
    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        if self.config.allowed_tools.is_empty() {
            return true; // Empty = allow all
        }

        self.config
            .allowed_tools
            .iter()
            .any(|pattern| glob::Pattern::new(pattern).map_or(false, |p| p.matches(tool_name)))
    }

    /// Get list of discovered tools
    pub fn tools(&self) -> &[McpToolDef] {
        &self.tools
    }

    /// Get server name
    pub fn name(&self) -> &str {
        &self.config.name
    }
}
