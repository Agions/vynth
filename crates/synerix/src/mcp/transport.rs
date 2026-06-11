//! MCP Transport trait
#![allow(dead_code)]

use crate::error::AppError;
use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;

/// Transport abstraction for MCP communication
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a request and wait for response
    async fn send_and_wait(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, AppError>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;

    /// Close the transport
    async fn close(&mut self) -> Result<(), AppError>;
}

/// Stdio transport (communicates via stdin/stdout of a child process)
pub struct StdioTransport {
    stdin_tx: Option<tokio::sync::mpsc::Sender<String>>,
    response_rx: Option<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<JsonRpcResponse>>>,
    child: Option<tokio::process::Child>,
    next_id: std::sync::atomic::AtomicU64,
}

impl StdioTransport {
    /// Spawn a child process and connect via stdio
    pub async fn connect(command: &str, args: &[String]) -> Result<Self, AppError> {
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| AppError::McpTransport(format!("Failed to spawn MCP server: {}", e)))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::McpTransport("Failed to get stdin handle from MCP server process".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::McpTransport(
                "Failed to get stdout handle from MCP server process".to_string(),
            )
        })?;

        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(32);
        let (resp_tx, resp_rx) = tokio::sync::mpsc::channel::<JsonRpcResponse>(32);

        // stdin writer task
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut stdin = stdin;
            while let Some(msg) = stdin_rx.recv().await {
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // stdout reader task
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&line) {
                    let _ = resp_tx.send(resp).await;
                }
                line.clear();
            }
        });

        let mut transport = Self {
            stdin_tx: Some(stdin_tx),
            response_rx: Some(tokio::sync::Mutex::new(resp_rx)),
            child: Some(child),
            next_id: std::sync::atomic::AtomicU64::new(1),
        };

        // Initialize handshake
        transport.initialize().await?;

        Ok(transport)
    }

    /// MCP initialize handshake
    async fn initialize(&mut self) -> Result<(), AppError> {
        let init_request = JsonRpcRequest::new(
            self.next_id(),
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "synerix",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        );

        let _response = self.send_and_wait(init_request).await?;

        // Send initialized notification
        let notification = JsonRpcRequest::new(self.next_id(), "notifications/initialized", None);

        if let Some(tx) = &self.stdin_tx {
            let mut msg = serde_json::to_string(&notification)?;
            msg.push('\n');
            let _ = tx.send(msg).await;
        }

        Ok(())
    }

    fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Test-only constructor — builds a connected-looking transport
    /// without spawning a real subprocess.
    #[cfg(test)]
    fn new_for_test() -> Self {
        use std::sync::atomic::AtomicU64;

        let (stdin_tx, _stdin_rx) = tokio::sync::mpsc::channel::<String>(32);
        let (_resp_tx, resp_rx) = tokio::sync::mpsc::channel::<JsonRpcResponse>(32);

        Self {
            stdin_tx: Some(stdin_tx),
            response_rx: Some(tokio::sync::Mutex::new(resp_rx)),
            child: None,
            next_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_and_wait(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, AppError> {
        if let Some(tx) = &self.stdin_tx {
            let mut msg = serde_json::to_string(&request)?;
            msg.push('\n');
            tx.send(msg)
                .await
                .map_err(|e| AppError::McpTransport(e.to_string()))?;
        }

        if let Some(rx) = &self.response_rx {
            // Wait with timeout
            match tokio::time::timeout(std::time::Duration::from_secs(30), async {
                // This is a simplified version — in production, use a proper
                // request/response correlation mechanism
                let mut guard = rx.lock().await;
                guard.recv().await
            })
            .await
            {
                Ok(Some(response)) => Ok(response),
                Ok(None) => Err(AppError::McpTransport("Channel closed".to_string())),
                Err(_) => Err(AppError::McpTransport("Request timed out".to_string())),
            }
        } else {
            Err(AppError::McpTransport("Not connected".to_string()))
        }
    }

    fn is_connected(&self) -> bool {
        self.stdin_tx.is_some()
    }

    async fn close(&mut self) -> Result<(), AppError> {
        self.stdin_tx = None;
        self.response_rx = None;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compile-time trait sanity ────────────────────────────────

    /// Verify McpTransport can be used as a trait object (dyn dispatch).
    /// If this compiles, the trait is object-safe and the Send+Sync bounds work.
    #[test]
    fn trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn McpTransport) {}
    }

    // ── StdioTransport::is_connected ─────────────────────────────

    #[tokio::test]
    async fn is_connected_true_when_stdin_tx_present() {
        let transport = StdioTransport::new_for_test();
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn is_connected_false_after_stdin_tx_cleared() {
        let mut transport = StdioTransport::new_for_test();
        transport.stdin_tx = None;
        assert!(!transport.is_connected());
    }

    // ── StdioTransport::close ────────────────────────────────────

    #[tokio::test]
    async fn close_clears_stdin_tx() {
        let mut transport = StdioTransport::new_for_test();
        assert!(transport.stdin_tx.is_some());

        transport.close().await.unwrap();

        assert!(transport.stdin_tx.is_none());
    }

    #[tokio::test]
    async fn close_clears_response_rx() {
        let mut transport = StdioTransport::new_for_test();
        assert!(transport.response_rx.is_some());

        transport.close().await.unwrap();

        assert!(transport.response_rx.is_none());
    }

    #[tokio::test]
    async fn close_makes_is_connected_false() {
        let mut transport = StdioTransport::new_for_test();
        assert!(transport.is_connected());

        transport.close().await.unwrap();

        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn close_without_child_succeeds() {
        // new_for_test sets child = None, so close should not panic
        let mut transport = StdioTransport::new_for_test();
        assert!(transport.child.is_none());
        assert!(transport.close().await.is_ok());
    }

    // ── next_id ──────────────────────────────────────────────────

    #[test]
    fn next_id_increments() {
        let transport = StdioTransport::new_for_test();
        let id1 = transport.next_id();
        let id2 = transport.next_id();
        let id3 = transport.next_id();
        // new_for_test starts at 1; fetch_add returns the old value
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }
}
