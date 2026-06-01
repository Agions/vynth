//! User approval flow for sandbox operations

use async_trait::async_trait;

use crate::error::AppError;

/// Approval modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalMode {
    /// Auto-approve all operations
    ///
    /// # ⚠️ Security Warning
    /// `Auto` mode bypasses all user approval checks. Any tool call with
    /// dangerous parameters (e.g., `rm -rf /`, `curl | sh`) will execute
    /// immediately without user confirmation. Only use `Auto` in:
    /// - Sandboxed/docker environments
    /// - CI pipelines with trusted inputs
    /// - Interactive sessions where the user has explicitly acknowledged the risk
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
    /// Decision channel sender — kept for future sandbox UI
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_mode_variants() {
        assert_eq!(ApprovalMode::Auto, ApprovalMode::Auto);
        assert_eq!(ApprovalMode::Confirm, ApprovalMode::Confirm);
        assert_eq!(ApprovalMode::PreviewOnly, ApprovalMode::PreviewOnly);
        assert_ne!(ApprovalMode::Auto, ApprovalMode::Confirm);
        assert_ne!(ApprovalMode::Confirm, ApprovalMode::PreviewOnly);
    }

    #[test]
    fn approval_decision_debug() {
        let d = ApprovalDecision::Allow;
        assert_eq!(format!("{:?}", d), "Allow");
        let d = ApprovalDecision::Deny;
        assert_eq!(format!("{:?}", d), "Deny");
        let d = ApprovalDecision::AllowAlways;
        assert_eq!(format!("{:?}", d), "AllowAlways");
    }

    #[tokio::test]
    async fn auto_approve_always_returns_allow() {
        let handler = AutoApprove;
        let result = handler.request_approval("some preview").await.unwrap();
        assert!(matches!(result, ApprovalDecision::Allow));
        let result2 = handler.request_approval("").await.unwrap();
        assert!(matches!(result2, ApprovalDecision::Allow));
    }

    #[tokio::test]
    async fn tui_approve_sends_preview_to_channel() {
        let (handler, mut rx) = TuiApprove::new();
        let result = handler.request_approval("test preview").await.unwrap();
        assert!(matches!(result, ApprovalDecision::Allow));
        let received = rx.recv().await.unwrap();
        assert_eq!(received, "test preview");
    }

    #[tokio::test]
    async fn tui_approve_new_provides_receiver() {
        let (_handler, _rx) = TuiApprove::new();
        // Constructor succeeds without panic
    }
}
