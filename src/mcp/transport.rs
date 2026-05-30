//! MCP Transport trait

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
            let msg = serde_json::to_string(&notification)?;
            let _ = tx.send(format!("{}\n", msg)).await;
        }

        Ok(())
    }

    fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_and_wait(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, AppError> {
        if let Some(tx) = &self.stdin_tx {
            let msg = serde_json::to_string(&request)?;
            tx.send(format!("{}\n", msg))
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
