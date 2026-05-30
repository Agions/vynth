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

        let content = tokio::fs::read_to_string(&full_path)
            .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_ctx(dir: &std::path::Path) -> ToolContext {
        ToolContext {
            working_dir: dir.to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_tool_name() {
        let tool = FileReadTool;
        assert_eq!(tool.name(), "file_read");
    }

    #[test]
    fn test_tool_schema() {
        let tool = FileReadTool;
        let schema = tool.schema();
        assert_eq!(schema["name"], "file_read");
        assert!(schema["parameters"]["properties"]["path"].is_object());
    }

    #[tokio::test]
    async fn test_read_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = FileReadTool;
        let args = json!({"path": "test.txt"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("1|line1"));
        assert!(result.output.contains("2|line2"));
        assert!(result.output.contains("3|line3"));
    }

    #[tokio::test]
    async fn test_read_with_offset() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = FileReadTool;
        let args = json!({"path": "test.txt", "offset": 2});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("2|line2"));
        assert!(result.output.contains("3|line3"));
        assert!(!result.output.contains("1|line1"));
    }

    #[tokio::test]
    async fn test_read_with_limit() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = FileReadTool;
        let args = json!({"path": "test.txt", "limit": 2});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("1|line1"));
        assert!(result.output.contains("2|line2"));
        assert!(!result.output.contains("3|line3"));
    }

    #[tokio::test]
    async fn test_read_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileReadTool;
        let args = json!({});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileReadTool;
        let args = json!({"path": "nonexistent.txt"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("empty.txt"), "").unwrap();

        let tool = FileReadTool;
        let args = json!({"path": "empty.txt"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.is_empty());
    }
}
