//! Agent — core reasoning loop + multi-agent orchestration

pub mod agent_loop;
pub mod context;
pub mod custom;
pub mod multi;
pub mod prompt;
pub mod roles;
pub mod tool_dispatcher;

pub use agent_loop::run_agent_loop;
pub use context::{ContextManager, TokenBudget};
pub use custom::{CustomAgentDef, CustomAgentRegistry};
pub use multi::{
    AgentBus, AgentConfig, AgentId, AgentInstance, AgentMessage, AgentStatus, AgentSwarm,
    AgentSwarmEvent, MessageType,
};
pub use prompt::{
    build_role_prompt, build_system_prompt, build_tool_aware_prompt, default_system_prompt,
    ProjectContext,
};
pub use roles::{AgentCapabilities, AgentRole};
