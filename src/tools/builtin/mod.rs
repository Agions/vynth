//! Built-in tools

pub mod file_read;
pub mod file_write;
pub mod shell_exec;
pub mod search;
pub mod patch;

use std::sync::Arc;
use crate::tools::registry::ToolRegistry;

/// Register all built-in tools
pub fn register_builtins(registry: &mut ToolRegistry) {
    registry.register(Arc::new(file_read::FileReadTool));
    registry.register(Arc::new(file_write::FileWriteTool));
    registry.register(Arc::new(shell_exec::ShellExecTool));
    registry.register(Arc::new(search::SearchTool));
    registry.register(Arc::new(patch::PatchTool));
}
