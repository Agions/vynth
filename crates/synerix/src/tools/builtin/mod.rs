//! Built-in tools
// TODO: Infrastructure awaiting main-loop integration
#![allow(dead_code)]
// TODO: Some re-exports unused until integration is complete
#![allow(unused_imports)]

pub mod file_read;
pub mod file_write;
pub mod patch;
pub mod search;
pub mod shell_exec;

use crate::tools::registry::ToolRegistry;
use std::sync::Arc;

pub use file_read::FileReadTool;
pub use file_write::FileWriteTool;
pub use patch::PatchTool;
pub use search::SearchTool;
pub use shell_exec::ShellExecTool;

/// Register all built-in tools
pub fn register_builtins(registry: &mut ToolRegistry) {
    registry.register(Arc::new(file_read::FileReadTool));
    registry.register(Arc::new(file_write::FileWriteTool));
    registry.register(Arc::new(shell_exec::ShellExecTool));
    registry.register(Arc::new(search::SearchTool));
    registry.register(Arc::new(patch::PatchTool));
}
