//! File write tool (sandbox — atomic replace)

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::tools::trait_def::{Tool, ToolContext, ToolResult};

pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "file_write",
            "description": "Write content to a file (creates parent directories). Uses atomic write (write-then-rename).",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }
        })
    }

    fn requires_approval(&self, _args: &Value) -> bool {
        true // File writes always require approval
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError> {
        let path = args["path"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'path'".to_string()))?;
        let content = args["content"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'content'".to_string()))?;

        let full_path = ctx.working_dir.join(path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Atomic write: write to temp → rename
        let tmp_path = full_path.with_extension("tmp.syncode");
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &full_path)?;

        let line_count = content.lines().count();
        let byte_size = content.len();

        Ok(ToolResult {
            output: format!("Wrote {} ({} lines, {} bytes)", path, line_count, byte_size),
            is_error: false,
            preview: None,
        })
    }
}
