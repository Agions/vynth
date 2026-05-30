//! Message bus for inter-agent communication

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::error::AppError;

use super::types::{AgentId, AgentMessage, MessageType};

/// Message bus for inter-agent communication
pub struct AgentBus {
    pub(crate) channels: HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>,
}

impl AgentBus {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent_id: AgentId, tx: mpsc::UnboundedSender<AgentMessage>) {
        self.channels.insert(agent_id, tx);
    }

    pub fn unregister(&mut self, agent_id: &str) {
        self.channels.remove(agent_id);
    }

    pub fn send(&self, msg: AgentMessage) -> Result<(), AppError> {
        if let Some(tx) = self.channels.get(&msg.to) {
            tx.send(msg)
                .map_err(|e| AppError::ExecutionFailed(format!("Send failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn broadcast(&self, from: &str, content: &str) -> Result<(), AppError> {
        let from_owned = from.to_string();
        let content_owned = content.to_string();
        for (id, tx) in &self.channels {
            if id != from {
                let msg = AgentMessage {
                    from: from_owned.clone(),
                    to: id.clone(),
                    content: content_owned.clone(),
                    msg_type: MessageType::Broadcast,
                    task_id: None,
                };
                let _ = tx.send(msg);
            }
        }
        Ok(())
    }

    /// Async send with timeout. Returns error if the send fails or times out.
    pub async fn send_async(&self, msg: AgentMessage, timeout: Duration) -> Result<(), AppError> {
        let to = msg.to.clone();
        if let Some(tx) = self.channels.get(&to) {
            let tx = tx.clone();
            tokio::time::timeout(timeout, async move {
                tx.send(msg)
                    .map_err(|e| AppError::ExecutionFailed(format!("Send failed: {}", e)))
            })
            .await
            .map_err(|_| AppError::ExecutionFailed(format!("Send to {} timed out", to)))?
        } else {
            Err(AppError::ExecutionFailed(format!(
                "Agent {} not registered",
                to
            )))
        }
    }

    /// Async broadcast with timeout. Collects results from all sends.
    pub async fn broadcast_async(
        &self,
        from: &str,
        content: &str,
        timeout: Duration,
    ) -> Result<(), AppError> {
        let mut futs = Vec::new();
        for (id, tx) in &self.channels {
            if id != from {
                let msg = AgentMessage {
                    from: from.to_string(),
                    to: id.clone(),
                    content: content.to_string(),
                    msg_type: MessageType::Broadcast,
                    task_id: None,
                };
                let tx = tx.clone();
                futs.push(async move { tx.send(msg) });
            }
        }
        // Drive all sends concurrently with timeout
        let results = tokio::time::timeout(timeout, futures::future::join_all(futs))
            .await
            .map_err(|_| AppError::ExecutionFailed("Broadcast timed out".to_string()))?;
        for result in results {
            result
                .map_err(|e| AppError::ExecutionFailed(format!("Broadcast send failed: {}", e)))?;
        }
        Ok(())
    }
}
