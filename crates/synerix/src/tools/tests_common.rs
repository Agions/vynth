//! Shared test helpers for builtin tools.

use crate::tools::traits::ToolContext;

/// Create a test ToolContext pointing to a temp directory.
pub fn test_ctx(dir: &std::path::Path) -> ToolContext {
    ToolContext {
        working_dir: dir.to_path_buf(),
        ..Default::default()
    }
}
