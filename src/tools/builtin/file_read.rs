//! File read tool

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::tools::trait_def::{Tool, ToolContext, ToolResult};

pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "file_read",
            "description": "Read the contents of a file",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line number to start reading from (1-indexed)",
                        "default": 1
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of lines to read",
                        "default": 500
                    }
                },
                "required": ["path"]
            }
        })
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError> {
        let path = args["path"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'path'".to_string()))?;

        let offset = args["offset"].as_u64().unwrap_or(1) as usize;
        let limit = args["limit"].as_u64().unwrap_or(500) as usize;

        let full_path = ctx.working_dir.join(path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| AppError::ExecutionFailed(format!("Failed to read {}: {}", path, e)))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = (offset - 1).min(lines.len());
        let end = (start + limit).min(lines.len());

        let output: String = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{}|{}", start + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult {
            output,
            is_error: false,
            preview: None,
        })
    }
}
