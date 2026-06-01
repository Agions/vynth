//! File search tool (ripgrep-like)

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::tools::traits::{Tool, ToolContext, ToolResult};

pub struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "search",
            "description": "Search for files by name pattern or content regex. Returns matching file paths and line numbers.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex for content, glob for filenames)"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["content", "filename"],
                        "description": "Search mode: 'content' searches inside files, 'filename' matches file names",
                        "default": "content"
                    },
                    "file_glob": {
                        "type": "string",
                        "description": "Optional file glob filter (e.g. '*.rs', '*.py')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 50)",
                        "default": 50
                    }
                },
                "required": ["pattern"]
            }
        })
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'pattern'".to_string()))?;
        let mode = args["mode"].as_str().unwrap_or("content");
        let file_glob = args["file_glob"].as_str();
        let max_results = args["max_results"].as_u64().unwrap_or(50) as usize;

        let mut cmd_args: Vec<String> = Vec::new();

        match mode {
            "content" => {
                cmd_args.push("--no-heading".to_string());
                cmd_args.push("--line-number".to_string());
                cmd_args.push("--color=never".to_string());
                if let Some(glob) = file_glob {
                    cmd_args.push("--glob".to_string());
                    cmd_args.push(glob.to_string());
                }
                cmd_args.push("--".to_string());
                cmd_args.push(pattern.to_string());
                cmd_args.push(".".to_string());
            }
            "filename" => {
                // Use find with name pattern
                let output = tokio::process::Command::new("find")
                    .arg(".")
                    .arg("-name")
                    .arg(pattern)
                    .arg("-maxdepth")
                    .arg("10")
                    .current_dir(&ctx.working_dir)
                    .output()
                    .await
                    .map_err(|e| AppError::ExecutionFailed(e.to_string()))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let results: Vec<&str> = stdout.lines().take(max_results).collect();

                return Ok(ToolResult {
                    output: results.join("\n"),
                    is_error: false,
                    preview: None,
                });
            }
            _ => {
                return Err(AppError::InvalidArgs(format!(
                    "Unknown search mode: {}",
                    mode
                )));
            }
        }

        let output = tokio::process::Command::new("rg")
            .args(&cmd_args)
            .current_dir(&ctx.working_dir)
            .output()
            .await
            .map_err(|e| {
                AppError::ExecutionFailed(format!("ripgrep not found or failed: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: Vec<&str> = stdout.lines().take(max_results).collect();

        Ok(ToolResult {
            output: if results.is_empty() {
                "No matches found.".to_string()
            } else {
                results.join("\n")
            },
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
        let tool = SearchTool;
        assert_eq!(tool.name(), "search");
    }

    #[test]
    fn test_tool_schema() {
        let tool = SearchTool;
        let schema = tool.schema();
        assert_eq!(schema["name"], "search");
        assert!(schema["parameters"]["properties"]["pattern"].is_object());
    }

    #[tokio::test]
    async fn test_content_search() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("test.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        let tool = SearchTool;
        let args = json!({"pattern": "main"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("main"));
    }

    #[tokio::test]
    async fn test_filename_search() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "").unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();

        let tool = SearchTool;
        let args = json!({"pattern": "*.rs", "mode": "filename"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("test.rs") || result.output.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_content_search_with_glob() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "hello world").unwrap();
        fs::write(dir.path().join("test.py"), "hello world").unwrap();

        let tool = SearchTool;
        let args = json!({"pattern": "hello", "file_glob": "*.rs"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        // Should only match .rs file
        assert!(result.output.contains("test.rs"));
    }

    #[tokio::test]
    async fn test_no_matches() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();

        let tool = SearchTool;
        let args = json!({"pattern": "zzzznonexistent"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("No matches"));
    }

    #[tokio::test]
    async fn test_invalid_mode() {
        let dir = tempfile::tempdir().unwrap();
        let tool = SearchTool;
        let args = json!({"pattern": "test", "mode": "invalid"});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_pattern() {
        let dir = tempfile::tempdir().unwrap();
        let tool = SearchTool;
        let args = json!({});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_max_results() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..10 {
            fs::write(dir.path().join(format!("f{}.txt", i)), "match_target").unwrap();
        }

        let tool = SearchTool;
        let args = json!({"pattern": "match_target", "max_results": 3});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        let line_count = result.output.lines().count();
        assert!(line_count <= 3);
    }
}
