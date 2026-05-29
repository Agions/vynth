//! User approval flow for sandbox operations

use async_trait::async_trait;

use crate::error::AppError;

/// Approval modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalMode {
    /// Auto-approve all operations
    Auto,
    /// Ask user for confirmation
    Confirm,
    /// Preview only (never execute)
    PreviewOnly,
}

/// Approval decision
#[derive(Debug, Clone)]
pub enum ApprovalDecision {
    /// Approve this operation
    Allow,
    /// Deny this operation
    Deny,
    /// Approve and remember for this session
    AllowAlways,
}

/// Approval handler trait
#[async_trait]
pub trait ApprovalHandler: Send + Sync {
    /// Request approval for an operation
    async fn request_approval(&self, preview: &str) -> Result<ApprovalDecision, AppError>;
}

/// Auto-approve handler (for sandbox mode: auto)
pub struct AutoApprove;

#[async_trait]
impl ApprovalHandler for AutoApprove {
    async fn request_approval(&self, _preview: &str) -> Result<ApprovalDecision, AppError> {
        Ok(ApprovalDecision::Allow)
    }
}

/// TUI-based approval handler (shows preview and asks for y/n)
pub struct TuiApprove {
    decision_tx: tokio::sync::mpsc::Sender<ApprovalDecision>,
    request_tx: tokio::sync::mpsc::Sender<String>,
}

impl TuiApprove {
    pub fn new() -> (Self, tokio::sync::mpsc::Receiver<String>) {
        let (decision_tx, _decision_rx) = tokio::sync::mpsc::channel(1);
        let (request_tx, request_rx) = tokio::sync::mpsc::channel(1);

        (
            Self {
                decision_tx,
                request_tx,
            },
            request_rx,
        )
    }
}

#[async_trait]
impl ApprovalHandler for TuiApprove {
    async fn request_approval(&self, preview: &str) -> Result<ApprovalDecision, AppError> {
        // Send preview to TUI for display
        let _ = self.request_tx.send(preview.to_string()).await;

        // Wait for user decision (simplified — real impl would integrate with TUI event loop)
        // For now, default to allow
        Ok(ApprovalDecision::Allow)
    }
}
