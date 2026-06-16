//! MCP Client — single server connection
//!
//! Performance: caches compiled glob patterns for tool permission checks
//! to avoid recompilation on every tool call.

use crate::config::McpServerConfig;
use crate::error::AppError;
use crate::mcp::transport::{McpTransport, StdioTransport};
use crate::mcp::types::*;

/// Single MCP Server connection with cached permission patterns
pub struct McpClient {
    config: McpServerConfig,
    transport: Option<Box<dyn McpTransport>>,
    tools: Vec<McpToolDef>,
    /// Pre-compiled glob patterns for tool permission checks
    /// (None = allow all, empty = deny all)
    allowed_patterns: Option<Vec<glob::Pattern>>,
    /// Number of reconnections (for monitoring)
    reconnect_count: u64,
}

impl McpClient {
    /// Connect to an MCP server
    pub async fn connect(config: McpServerConfig) -> Result<Self, AppError> {
        // Pre-compile glob patterns once at connection time
        let allowed_patterns = if config.allowed_tools.is_empty() {
            None // None = allow all
        } else {
            let patterns: Vec<glob::Pattern> = config
                .allowed_tools
                .iter()
                .filter_map(|p| glob::Pattern::new(p).ok())
                .collect();
            Some(patterns)
        };

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
            allowed_patterns,
            reconnect_count: 0,
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
        // Permission check (uses pre-compiled patterns)
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
    ///
    /// Uses pre-compiled glob patterns (O(n) match, no recompilation)
    #[inline]
    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        match &self.allowed_patterns {
            None => true, // Empty config = allow all
            Some(patterns) => patterns.iter().any(|p| p.matches(tool_name)),
        }
    }

    /// Get list of discovered tools
    pub fn tools(&self) -> &[McpToolDef] {
        &self.tools
    }

    /// Get server name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get reconnect count (for health monitoring)
    #[allow(dead_code)]
    pub fn reconnect_count(&self) -> u64 {
        self.reconnect_count
    }

    /// Reconnect to the MCP server (explicit, for manual use or health check)
    #[allow(dead_code)]
    pub async fn reconnect(&mut self) -> Result<(), AppError> {
        // Close existing transport
        if let Some(mut transport) = self.transport.take() {
            let _ = transport.close().await;
        }

        // Re-spawn the child process
        let transport: Box<dyn McpTransport> = match &self.config.transport {
            crate::config::McpTransport::Stdio { command, args } => {
                Box::new(StdioTransport::connect(command, args).await?)
            }
            crate::config::McpTransport::Http { .. } => {
                return Err(AppError::McpTransport("HTTP reconnect not supported".to_string()));
            }
        };

        self.transport = Some(transport);
        self.reconnect_count += 1;

        // Re-discover tools
        self.discover_tools().await
    }

    /// Test-only constructor for unit testing without a real subprocess
    #[cfg(test)]
    pub(crate) fn new_for_test(name: &str, allowed_tools: Vec<String>) -> Self {
        use std::collections::HashMap;

        let config = McpServerConfig {
            name: name.to_string(),
            transport: crate::config::McpTransport::Stdio {
                command: "noop".to_string(),
                args: vec![],
            },
            allowed_tools: allowed_tools.clone(),
            env: HashMap::new(),
            cwd: None,
            auto_reconnect: true,
            timeout_secs: 30,
        };

        let allowed_patterns = if allowed_tools.is_empty() {
            None
        } else {
            let patterns: Vec<glob::Pattern> = allowed_tools
                .iter()
                .filter_map(|p| glob::Pattern::new(p).ok())
                .collect();
            Some(patterns)
        };

        Self {
            config,
            transport: None,
            tools: Vec::new(),
            allowed_patterns,
            reconnect_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_tool_allowed ──────────────────────────────────────────

    #[test]
    fn is_tool_allowed_none_patterns_allows_all() {
        let client = McpClient::new_for_test("test", vec![]);
        assert!(client.is_tool_allowed("anything"));
        assert!(client.is_tool_allowed("read_file"));
        assert!(client.is_tool_allowed("special/tool"));
    }

    #[test]
    fn is_tool_allowed_empty_vec_allows_all() {
        let client = McpClient::new_for_test("test", vec![]);
        assert!(client.is_tool_allowed("any_tool"));
    }

    #[test]
    fn is_tool_allowed_exact_match() {
        let client = McpClient::new_for_test("test", vec!["read_file".to_string()]);
        assert!(client.is_tool_allowed("read_file"));
        assert!(!client.is_tool_allowed("write_file"));
        assert!(!client.is_tool_allowed("read"));
        assert!(!client.is_tool_allowed("read_file_extra"));
    }

    #[test]
    fn is_tool_allowed_star_glob() {
        let client = McpClient::new_for_test("test", vec!["read_*".to_string()]);
        assert!(client.is_tool_allowed("read_file"));
        assert!(client.is_tool_allowed("read_dir"));
        assert!(client.is_tool_allowed("read_"));
        assert!(!client.is_tool_allowed("write_file"));
        assert!(!client.is_tool_allowed("read"));
    }

    #[test]
    fn is_tool_allowed_question_mark_glob() {
        let client = McpClient::new_for_test("test", vec!["tool_?".to_string()]);
        assert!(client.is_tool_allowed("tool_a"));
        assert!(client.is_tool_allowed("tool_1"));
        assert!(!client.is_tool_allowed("tool_ab"));
        assert!(!client.is_tool_allowed("tool_"));
    }

    #[test]
    fn is_tool_allowed_multiple_patterns() {
        let client = McpClient::new_for_test(
            "test",
            vec![
                "read_*".to_string(),
                "list_*".to_string(),
                "get_status".to_string(),
            ],
        );
        assert!(client.is_tool_allowed("read_file"));
        assert!(client.is_tool_allowed("list_tools"));
        assert!(client.is_tool_allowed("get_status"));
        assert!(!client.is_tool_allowed("write_file"));
        assert!(!client.is_tool_allowed("delete_all"));
    }

    #[test]
    fn is_tool_allowed_no_match_denies() {
        let client = McpClient::new_for_test("test", vec!["specific_tool".to_string()]);
        assert!(!client.is_tool_allowed("other_tool"));
        assert!(!client.is_tool_allowed("specific"));
        assert!(!client.is_tool_allowed("specific_tool_extra"));
    }

    #[test]
    fn is_tool_allowed_bracket_glob() {
        let client = McpClient::new_for_test("test", vec!["tool_[abc]".to_string()]);
        assert!(client.is_tool_allowed("tool_a"));
        assert!(client.is_tool_allowed("tool_b"));
        assert!(client.is_tool_allowed("tool_c"));
        assert!(!client.is_tool_allowed("tool_d"));
        assert!(!client.is_tool_allowed("tool_ab"));
    }

    // ── accessors ────────────────────────────────────────────────

    #[test]
    fn name_returns_config_name() {
        let client = McpClient::new_for_test("my_server", vec![]);
        assert_eq!(client.name(), "my_server");
    }

    #[test]
    fn tools_initially_empty() {
        let client = McpClient::new_for_test("test", vec![]);
        assert!(client.tools().is_empty());
    }

    #[test]
    fn reconnect_count_initially_zero() {
        let client = McpClient::new_for_test("test", vec![]);
        assert_eq!(client.reconnect_count(), 0);
    }
}