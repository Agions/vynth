//! Agent — core reasoning loop

pub mod agloop;
pub mod context;
pub mod prompt;

pub use agloop::run_agent_loop;
pub use context::{ContextManager, TokenBudget};
