//! Agent — core reasoning loop + multi-agent orchestration

pub mod agloop;
pub mod context;
pub mod multi;
pub mod prompt;
pub mod roles;

pub use agloop::run_agent_loop;
pub use context::{ContextManager, TokenBudget};
pub use multi::{AgentBus, AgentConfig, AgentId, AgentInstance, AgentMessage, AgentStatus, AgentSwarm};
pub use roles::{AgentCapabilities, AgentRole};
