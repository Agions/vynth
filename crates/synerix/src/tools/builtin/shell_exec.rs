//! Shell execution tool (sandbox — dangerous command detection)
#![allow(dead_code)]

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::AppError;
use crate::tools::traits::{Tool, ToolContext, ToolResult};

pub struct ShellExecTool;

/// Dangerous command patterns that require approval
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r /",
    "mkfs",
    "dd if=",
    "> /dev/",
    ":(){ :|:& };:",
    "chmod -R 777",
    "curl.*|.*sh",
    "wget.*|.*sh",
];

#[async_trait]
impl Tool for ShellExecTool {
    fn name(&self) -> &str {
        "shell_exec"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "shell_exec",
            "description": "Execute a shell command in the working directory. Output includes stdout and stderr.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    },
                    "timeout_secs": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 120)",
                        "default": 120
                    }
                },
                "required": ["command"]
            }
        })
    }

    fn requires_approval(&self, args: &Value) -> bool {
        let cmd = args["command"].as_str().unwrap_or("");
        DANGEROUS_PATTERNS
            .iter()
            .any(|pattern| cmd.contains(pattern))
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult, AppError> {
        let command = args["command"]
            .as_str()
            .ok_or(AppError::InvalidArgs("missing 'command'".to_string()))?;

        let timeout_secs = args["timeout_secs"].as_u64().unwrap_or(120);

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&ctx.working_dir)
                .output(),
        )
        .await
        .map_err(|_| AppError::ExecutionFailed("Command timed out".to_string()))?
        .map_err(|e| AppError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let combined = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n[stderr]\n{}", stdout, stderr)
        };

        // Trim very long output
        let output_text = if combined.len() > 50_000 {
            format!(
                "{}... (truncated, {} bytes total)",
                &combined[..50_000],
                combined.len()
            )
        } else {
            combined
        };

        Ok(ToolResult {
            output: output_text,
            is_error: !output.status.success(),
            preview: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx(dir: &std::path::Path) -> ToolContext {
        ToolContext {
            working_dir: dir.to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_tool_name() {
        let tool = ShellExecTool;
        assert_eq!(tool.name(), "shell_exec");
    }

    #[test]
    fn test_tool_schema() {
        let tool = ShellExecTool;
        let schema = tool.schema();
        assert_eq!(schema["name"], "shell_exec");
        assert!(schema["parameters"]["properties"]["command"].is_object());
    }

    #[test]
    fn test_requires_approval_safe_commands() {
        let tool = ShellExecTool;
        assert!(!tool.requires_approval(&json!({"command": "ls -la"})));
        assert!(!tool.requires_approval(&json!({"command": "echo hello"})));
        assert!(!tool.requires_approval(&json!({"command": "cargo build"})));
    }

    #[test]
    fn test_requires_approval_dangerous_commands() {
        let tool = ShellExecTool;
        assert!(tool.requires_approval(&json!({"command": "rm -rf /tmp/data"})));
        assert!(tool.requires_approval(&json!({"command": "rm -r /"})));
        assert!(tool.requires_approval(&json!({"command": "chmod -R 777 /var"})));
    }

    #[tokio::test]
    async fn test_execute_simple_command() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({"command": "echo hello"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_command_with_stderr() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({"command": "echo error >&2"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("[stderr]"));
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({"command": "false"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_execute_missing_command() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({});
        let result = tool.execute(args, &test_ctx(dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_working_dir() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({"command": "pwd"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains(dir.path().to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_execute_multiline_output() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ShellExecTool;
        let args = json!({"command": "printf 'line1\nline2\nline3\n'"});
        let result = tool.execute(args, &test_ctx(dir.path())).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("line1"));
        assert!(result.output.contains("line2"));
        assert!(result.output.contains("line3"));
    }
}
