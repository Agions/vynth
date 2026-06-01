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

impl Default for AgentBus {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(from: &str, to: &str, content: &str) -> AgentMessage {
        AgentMessage {
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            msg_type: MessageType::Request,
            task_id: None,
        }
    }

    #[test]
    fn test_new_bus_is_empty() {
        let bus = AgentBus::new();
        assert!(bus.channels.is_empty());
    }

    #[test]
    fn test_register_agent() {
        let mut bus = AgentBus::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);
        assert_eq!(bus.channels.len(), 1);
    }

    #[test]
    fn test_unregister_agent() {
        let mut bus = AgentBus::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);
        bus.unregister("agent_0");
        assert!(bus.channels.is_empty());
    }

    #[test]
    fn test_unregister_nonexistent() {
        let mut bus = AgentBus::new();
        bus.unregister("nonexistent"); // should not panic
        assert!(bus.channels.is_empty());
    }

    #[test]
    fn test_send_to_registered() {
        let mut bus = AgentBus::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);

        let msg = make_msg("agent_1", "agent_0", "hello");
        bus.send(msg).unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "hello");
        assert_eq!(received.from, "agent_1");
    }

    #[test]
    fn test_send_to_unregistered_is_ok() {
        let bus = AgentBus::new();
        let msg = make_msg("agent_1", "nonexistent", "hello");
        // send to unregistered agent returns Ok (silent drop)
        assert!(bus.send(msg).is_ok());
    }

    #[test]
    fn test_broadcast_excludes_sender() {
        let mut bus = AgentBus::new();
        let (tx0, mut rx0) = mpsc::unbounded_channel();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx0);
        bus.register("agent_1".into(), tx1);
        bus.register("agent_2".into(), tx2);

        bus.broadcast("agent_0", "update").unwrap();

        // agent_0 (sender) should NOT receive
        assert!(rx0.try_recv().is_err());
        // agent_1 and agent_2 should receive
        let msg1 = rx1.try_recv().unwrap();
        assert_eq!(msg1.content, "update");
        assert_eq!(msg1.from, "agent_0");
        assert!(matches!(msg1.msg_type, MessageType::Broadcast));

        let msg2 = rx2.try_recv().unwrap();
        assert_eq!(msg2.content, "update");
    }

    #[tokio::test]
    async fn test_send_async_success() {
        let mut bus = AgentBus::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);

        let msg = make_msg("agent_1", "agent_0", "async hello");
        bus.send_async(msg, Duration::from_secs(1)).await.unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "async hello");
    }

    #[tokio::test]
    async fn test_send_async_unregistered() {
        let bus = AgentBus::new();
        let msg = make_msg("agent_1", "nonexistent", "hello");
        let result = bus.send_async(msg, Duration::from_secs(1)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_async_success() {
        let mut bus = AgentBus::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), mpsc::unbounded_channel().0);
        bus.register("agent_1".into(), tx1);
        bus.register("agent_2".into(), tx2);

        bus.broadcast_async("agent_0", "async update", Duration::from_secs(1))
            .await
            .unwrap();

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }
}
