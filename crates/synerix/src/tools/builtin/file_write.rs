//! File write tool (sandbox — atomic replace)

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::tools::traits::{Tool, ToolContext, ToolResult};

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
            tokio::fs::create_dir_all(parent).await?;
        }

        // Atomic write: write to temp → rename
        let tmp_path = full_path.with_extension("tmp.synerix");
        tokio::fs::write(&tmp_path, content).await?;
        tokio::fs::rename(&tmp_path, &full_path).await?;

        let line_count = content.lines().count();
        let byte_size = content.len();

        Ok(ToolResult {
            output: format!("Wrote {} ({} lines, {} bytes)", path, line_count, byte_size),
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
        let tool = FileWriteTool;
        assert_eq!(tool.name(), "file_write");
    }

    #[test]
    fn test_tool_schema() {
        let tool = FileWriteTool;
        let schema = tool.schema();
        assert_eq!(schema["name"], "file_write");
        assert!(schema["parameters"]["properties"]["path"].is_object());
        assert!(schema["parameters"]["properties"]["content"].is_object());
    }

    #[test]
    fn test_requires_approval() {
        let tool = FileWriteTool;
        assert!(tool.requires_approval(&json!({})));
    }

    #[tokio::test]
    async fn test_write_file() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let args = json!({"path": "output.txt", "content": "hello world\n"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("output.txt"));
        assert!(result.output.contains("1 lines"));

        let content = fs::read_to_string(dir.path().join("output.txt")).unwrap();
        assert_eq!(content, "hello world\n");
    }

    #[tokio::test]
    async fn test_write_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let args = json!({"path": "deep/nested/file.txt", "content": "test"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);

        let content = fs::read_to_string(dir.path().join("deep/nested/file.txt")).unwrap();
        assert_eq!(content, "test");
    }

    #[tokio::test]
    async fn test_write_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("existing.txt"), "old content").unwrap();

        let tool = FileWriteTool;
        let args = json!({"path": "existing.txt", "content": "new content"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);

        let content = fs::read_to_string(dir.path().join("existing.txt")).unwrap();
        assert_eq!(content, "new content");
    }

    #[tokio::test]
    async fn test_write_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let args = json!({"content": "test"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_missing_content() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let args = json!({"path": "test.txt"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_multiline() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let content = "line1\nline2\nline3\n";
        let args = json!({"path": "multi.txt", "content": content});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("3 lines"));

        let written = fs::read_to_string(dir.path().join("multi.txt")).unwrap();
        assert_eq!(written, content);
    }

    #[tokio::test]
    async fn test_write_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let tool = FileWriteTool;
        let args = json!({"path": "empty.txt", "content": ""});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);

        let content = fs::read_to_string(dir.path().join("empty.txt")).unwrap();
        assert!(content.is_empty());
    }
}
