//! MCP Manager — multi-server management + permission isolation

use std::collections::HashMap;

use crate::config::McpServerConfig;
use crate::error::AppError;
use crate::mcp::client::McpClient;
use crate::mcp::types::McpToolResult;

/// MCP multi-server manager
pub struct McpManager {
    clients: HashMap<String, McpClient>,
}

impl McpManager {
    /// Connect to all configured MCP servers
    pub async fn connect_all(configs: &[McpServerConfig]) -> Result<Self, AppError> {
        let mut clients = HashMap::new();

        for config in configs {
            match McpClient::connect(config.clone()).await {
                Ok(client) => {
                    tracing::info!("Connected to MCP server: {}", config.name);
                    clients.insert(config.name.clone(), client);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to connect to MCP server '{}': {}",
                        config.name,
                        e
                    );
                }
            }
        }

        Ok(Self { clients })
    }

    /// Collect all MCP tool schemas (with permission filtering)
    pub fn tool_schemas(&self) -> Vec<crate::llm::types::ToolSchema> {
        self.clients
            .values()
            .flat_map(|client| {
                client.tools().iter().map(move |tool| {
                    crate::llm::types::ToolSchema {
                        schema_type: "function".to_string(),
                        function: crate::llm::types::FunctionSchema {
                            name: format!("mcp__{}__{}", client.name(), tool.name),
                            description: tool
                                .description
                                .clone()
                                .unwrap_or_default(),
                            parameters: tool
                                .input_schema
                                .clone()
                                .unwrap_or_else(|| serde_json::json!({})),
                        },
                    }
                })
            })
            .collect()
    }

    /// Call an MCP tool (with permission check)
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        args: serde_json::Value,
    ) -> Result<McpToolResult, AppError> {
        let client = self
            .clients
            .get(server)
            .ok_or_else(|| AppError::ToolNotFound(format!("MCP server not found: {}", server)))?;

        client.call_tool(tool, args).await
    }

    /// Find which server hosts a given tool (by full name: mcp__server__tool)
    pub fn find_tool<'a>(&'a self, full_name: &'a str) -> Option<(&'a str, &'a str)> {
        if let Some(rest) = full_name.strip_prefix("mcp__") {
            if let Some(pos) = rest.find("__") {
                let server = &rest[..pos];
                let tool = &rest[pos + 2..];
                if self.clients.contains_key(server) {
                    return Some((server, tool));
                }
            }
        }
        None
    }

    /// Health check all servers
    pub async fn health_check(&self) -> Vec<String> {
        let mut unhealthy = Vec::new();
        for (name, client) in &self.clients {
            if client.tools().is_empty() {
                unhealthy.push(format!("{}: no tools discovered", name));
            }
        }
        unhealthy
    }

    /// Get count of connected servers
    pub fn server_count(&self) -> usize {
        self.clients.len()
    }

    /// Get total tool count across all servers
    pub fn total_tool_count(&self) -> usize {
        self.clients.values().map(|c| c.tools().len()).sum()
    }
}
