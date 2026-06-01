//! Multi-agent orchestration

mod bus;
mod instance;
mod swarm;
mod types;

pub use bus::AgentBus;
pub use instance::AgentInstance;
pub use swarm::AgentSwarm;
pub use types::{AgentConfig, AgentId, AgentMessage, AgentStatus, AgentSwarmEvent, MessageType};

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::agent::roles::AgentRole;

    use super::*;

    #[test]
    fn test_agent_config() {
        let config = AgentConfig::new(AgentRole::Coder, "test-coder");
        assert_eq!(config.name, "test-coder");
        assert_eq!(config.role, AgentRole::Coder);
        assert_eq!(config.max_turns, 15);
    }

    #[test]
    fn test_agent_instance() {
        let config = AgentConfig::new(AgentRole::Reviewer, "reviewer-1");
        let instance = AgentInstance::new("agent_0".into(), config);
        assert_eq!(instance.status, AgentStatus::Idle);
        assert!(instance.system_prompt().contains("reviewer"));
    }

    #[test]
    fn test_agent_bus() {
        let mut bus = AgentBus::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        bus.register("agent_0".into(), tx);
        assert!(bus.channels.contains_key("agent_0"));
    }

    #[tokio::test]
    async fn test_swarm_spawn() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        assert_eq!(id, "agent_0");
        assert_eq!(swarm.agent_status(&id), Some(AgentStatus::Idle));
        assert_eq!(swarm.agent_role(&id), Some(AgentRole::Coder));
    }

    #[tokio::test]
    async fn test_swarm_coordinate() {
        let mut swarm = AgentSwarm::new();
        let id1 = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        let id2 = swarm.spawn_agent(AgentConfig::new(AgentRole::Reviewer, "reviewer"));

        // Use run_task to exercise the sync placeholder path
        let result = swarm.run_task(&id1, "Review this code").await.unwrap();
        assert!(result.contains("coder"));
        assert!(result.contains("Review this code"));
        let result2 = swarm.run_task(&id2, "Review this code").await.unwrap();
        assert!(result2.contains("reviewer"));
        assert!(result2.contains("Review this code"));
    }

    #[tokio::test]
    async fn test_swarm_run_task() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Tester, "tester"));

        let result = swarm.run_task(&id, "Write tests").await.unwrap();
        assert!(result.contains("tester"));
    }

    #[tokio::test]
    async fn test_remove_agent() {
        let mut swarm = AgentSwarm::new();
        let id = swarm.spawn_agent(AgentConfig::new(AgentRole::Coder, "coder"));
        assert!(swarm.agent_status(&id).is_some());

        let removed_config = swarm.remove_agent(&id);
        assert!(removed_config.is_some());
        assert_eq!(removed_config.unwrap().name, "coder");
        assert!(swarm.agent_status(&id).is_none());
        assert!(swarm.agent_ids().is_empty());
    }

    #[test]
    fn test_agent_stop_restart() {
        let config = AgentConfig::new(AgentRole::Coder, "coder");
        let mut instance = AgentInstance::new("agent_0".into(), config);
        assert_eq!(instance.status, AgentStatus::Idle);

        instance.stop();
        assert_eq!(instance.status, AgentStatus::Done);

        instance.restart();
        assert_eq!(instance.status, AgentStatus::Idle);
    }
}
