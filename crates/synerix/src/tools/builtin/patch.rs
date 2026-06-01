//! Patch tool — apply unified diffs

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::tools::traits::{Tool, ToolContext, ToolResult};

pub struct PatchTool;

#[async_trait]
impl Tool for PatchTool {
    fn name(&self) -> &str {
        "patch"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "patch",
            "description": "Apply a unified diff patch to a file. Supports find-and-replace style edits.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to patch"
                    },
                    "old_text": {
                        "type": "string",
                        "description": "Exact text to find and replace"
                    },
                    "new_text": {
                        "type": "string",
                        "description": "Replacement text"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace all occurrences (default: false)",
                        "default": false
                    }
                },
                "required": ["path", "old_text", "new_text"]
            }
        })
    }

    fn requires_approval(&self, _args: &Value) -> bool {
        true // All patches require approval
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError> {
        let path = args["path"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'path'".to_string()))?;
        let old_text = args["old_text"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'old_text'".to_string()))?;
        let new_text = args["new_text"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'new_text'".to_string()))?;
        let replace_all = args["replace_all"].as_bool().unwrap_or(false);

        let full_path = ctx.working_dir.join(path);

        let content = tokio::fs::read_to_string(&full_path)
            .await
            .map_err(|e| AppError::ExecutionFailed(format!("Failed to read {}: {}", path, e)))?;

        if !content.contains(old_text) {
            return Ok(ToolResult {
                output: format!("Old text not found in {}", path),
                is_error: true,
                preview: None,
            });
        }

        let count = content.matches(old_text).count();
        let new_content = if replace_all {
            content.replace(old_text, new_text)
        } else {
            content.replacen(old_text, new_text, 1)
        };

        // Atomic write
        let tmp_path = full_path.with_extension("tmp.synerix");
        tokio::fs::write(&tmp_path, &new_content).await?;
        tokio::fs::rename(&tmp_path, &full_path).await?;

        let replaced = if replace_all { count } else { 1 };

        Ok(ToolResult {
            output: format!("Patched {} — {} replacement(s) made", path, replaced),
            is_error: false,
            preview: Some(format!(
                "--- a/{}\n+++ b/{}\n@@ -old +new @@\n-{}\n+{}",
                path, path, old_text, new_text
            )),
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
        let tool = PatchTool;
        assert_eq!(tool.name(), "patch");
    }

    #[test]
    fn test_tool_schema() {
        let tool = PatchTool;
        let schema = tool.schema();
        assert_eq!(schema["name"], "patch");
        assert!(schema["parameters"]["properties"]["path"].is_object());
        assert!(schema["parameters"]["properties"]["old_text"].is_object());
        assert!(schema["parameters"]["properties"]["new_text"].is_object());
    }

    #[test]
    fn test_requires_approval() {
        let tool = PatchTool;
        assert!(tool.requires_approval(&json!({})));
    }

    #[tokio::test]
    async fn test_patch_single_replacement() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world\nhello again\n").unwrap();

        let tool = PatchTool;
        let args = json!({"path": "test.txt", "old_text": "hello", "new_text": "hi"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("1 replacement"));

        let content = fs::read_to_string(dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "hi world\nhello again\n");
    }

    #[tokio::test]
    async fn test_patch_replace_all() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world\nhello again\n").unwrap();

        let tool = PatchTool;
        let args =
            json!({"path": "test.txt", "old_text": "hello", "new_text": "hi", "replace_all": true});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("2 replacement"));

        let content = fs::read_to_string(dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "hi world\nhi again\n");
    }

    #[tokio::test]
    async fn test_patch_text_not_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world\n").unwrap();

        let tool = PatchTool;
        let args = json!({"path": "test.txt", "old_text": "nonexistent", "new_text": "new"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(result.is_error);
        assert!(result.output.contains("not found"));
    }

    #[tokio::test]
    async fn test_patch_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let tool = PatchTool;
        let args = json!({"old_text": "a", "new_text": "b"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_patch_missing_old_text() {
        let dir = tempfile::tempdir().unwrap();
        let tool = PatchTool;
        let args = json!({"path": "test.txt", "new_text": "b"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_patch_missing_new_text() {
        let dir = tempfile::tempdir().unwrap();
        let tool = PatchTool;
        let args = json!({"path": "test.txt", "old_text": "a"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_patch_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let tool = PatchTool;
        let args = json!({"path": "nonexistent.txt", "old_text": "a", "new_text": "b"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_patch_generates_preview() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}\n").unwrap();

        let tool = PatchTool;
        let args = json!({"path": "test.rs", "old_text": "main", "new_text": "run"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(result.preview.is_some());
        let preview = result.preview.unwrap();
        assert!(preview.contains("main"));
        assert!(preview.contains("run"));
    }
}
