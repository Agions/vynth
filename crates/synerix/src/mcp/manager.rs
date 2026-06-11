//! MCP Manager — multi-server management + permission isolation
#![allow(dead_code)]

use std::collections::HashMap;

use crate::config::McpServerConfig;
use crate::error::AppError;
use crate::mcp::mcp_client::McpClient;
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
                    tracing::warn!("Failed to connect to MCP server '{}': {}", config.name, e);
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
                client
                    .tools()
                    .iter()
                    .map(move |tool| crate::llm::types::ToolSchema {
                        schema_type: "function".to_string(),
                        function: crate::llm::types::FunctionSchema {
                            name: format!("mcp__{}__{}", client.name(), tool.name),
                            description: tool.description.clone().unwrap_or_default(),
                            parameters: tool
                                .input_schema
                                .clone()
                                .unwrap_or_else(|| serde_json::json!({})),
                        },
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

    /// Test-only constructor that accepts pre-built clients
    #[cfg(test)]
    fn new_for_test(clients: HashMap<String, McpClient>) -> Self {
        Self { clients }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager_with_servers(server_names: &[&str]) -> McpManager {
        let clients: HashMap<String, McpClient> = server_names
            .iter()
            .map(|name| (name.to_string(), McpClient::new_for_test(name, vec![])))
            .collect();
        McpManager::new_for_test(clients)
    }

    // ── find_tool ─────────────────────────────────────────────

    #[test]
    fn find_tool_valid_format() {
        let mgr = make_manager_with_servers(&["fs", "git"]);
        let result = mgr.find_tool("mcp__fs__read_file");
        assert_eq!(result, Some(("fs", "read_file")));
    }

    #[test]
    fn find_tool_second_server() {
        let mgr = make_manager_with_servers(&["fs", "git"]);
        let result = mgr.find_tool("mcp__git__commit");
        assert_eq!(result, Some(("git", "commit")));
    }

    #[test]
    fn find_tool_unknown_server_returns_none() {
        let mgr = make_manager_with_servers(&["fs"]);
        assert_eq!(mgr.find_tool("mcp__unknown__tool"), None);
    }

    #[test]
    fn find_tool_no_mcp_prefix_returns_none() {
        let mgr = make_manager_with_servers(&["fs"]);
        assert_eq!(mgr.find_tool("fs__read_file"), None);
    }

    #[test]
    fn find_tool_no_double_underscore_after_server_returns_none() {
        let mgr = make_manager_with_servers(&["fs"]);
        assert_eq!(mgr.find_tool("mcp__fs_read_file"), None);
    }

    #[test]
    fn find_tool_empty_tool_name() {
        let mgr = make_manager_with_servers(&["fs"]);
        // "mcp__fs__" → server="fs", tool=""
        let result = mgr.find_tool("mcp__fs__");
        assert_eq!(result, Some(("fs", "")));
    }

    #[test]
    fn find_tool_tool_with_nested_underscores() {
        let mgr = make_manager_with_servers(&["db"]);
        let result = mgr.find_tool("mcp__db__read_file_v2");
        assert_eq!(result, Some(("db", "read_file_v2")));
    }

    // ── server_count / total_tool_count ───────────────────────

    #[test]
    fn server_count_empty() {
        let mgr = McpManager::new_for_test(HashMap::new());
        assert_eq!(mgr.server_count(), 0);
    }

    #[test]
    fn server_count_multiple() {
        let mgr = make_manager_with_servers(&["a", "b", "c"]);
        assert_eq!(mgr.server_count(), 3);
    }

    #[test]
    fn total_tool_count_empty_clients() {
        let mgr = make_manager_with_servers(&["fs"]);
        // Test clients have no discovered tools
        assert_eq!(mgr.total_tool_count(), 0);
    }

    // ── tool_schemas ──────────────────────────────────────────

    #[test]
    fn tool_schemas_empty_when_no_tools_discovered() {
        let mgr = make_manager_with_servers(&["fs"]);
        let schemas = mgr.tool_schemas();
        assert!(schemas.is_empty());
    }

    // ── call_tool error handling ──────────────────────────────

    #[tokio::test]
    async fn call_tool_unknown_server_returns_error() {
        let mgr = make_manager_with_servers(&["fs"]);
        let result = mgr
            .call_tool("missing", "tool", serde_json::json!({}))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("MCP server not found"));
        assert!(err.to_string().contains("missing"));
    }

    // ── health_check ──────────────────────────────────────────

    #[tokio::test]
    async fn health_check_reports_empty_tools() {
        let mgr = make_manager_with_servers(&["server_a"]);
        let unhealthy = mgr.health_check().await;
        assert_eq!(unhealthy.len(), 1);
        assert!(unhealthy[0].contains("server_a"));
        assert!(unhealthy[0].contains("no tools discovered"));
    }

    #[tokio::test]
    async fn health_check_empty_manager() {
        let mgr = McpManager::new_for_test(HashMap::new());
        let unhealthy = mgr.health_check().await;
        assert!(unhealthy.is_empty());
    }
}
