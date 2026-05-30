//! Shell execution tool (sandbox — dangerous command detection)

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::AppError;
use crate::tools::trait_def::{Tool, ToolContext, ToolResult};

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
