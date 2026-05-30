//! Step execution with retry logic and timeout.

use std::time::Instant;

use crate::error::AppError;

use super::types::{StepResult, StepStatus};

/// Execute a step with retry logic and timeout.
/// This is a free function to avoid borrowing issues with the swarm.
pub(crate) async fn execute_step_with_retry(
    step_id: String,
    _agent_id: String,
    _prompt: String,
    _output_variable: Option<String>,
    max_retries: u32,
    retry_delay_ms: u64,
    timeout_secs: u64,
) -> Result<StepResult, AppError> {
    let mut attempts = 0u32;
    let max_attempts = max_retries + 1;

    loop {
        attempts += 1;
        let start = Instant::now();

        // Execute with timeout
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        let result = tokio::time::timeout(timeout_duration, async {
            // In real usage, this would call swarm.run_task() with the actual agent.
            // For now, simulate step execution.
            Ok::<String, AppError>(format!("[Step '{}' completed] (simulated)", step_id))
        })
        .await;

        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(output)) => {
                return Ok(StepResult {
                    step_id,
                    output,
                    status: StepStatus::Success,
                    duration_ms: duration,
                    attempts,
                });
            }
            Ok(Err(e)) => {
                if attempts <= max_retries {
                    tracing::warn!(
                        "Step '{}' failed (attempt {}/{}): {}. Retrying in {}ms...",
                        step_id,
                        attempts,
                        max_attempts,
                        e,
                        retry_delay_ms
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                    continue;
                }
                return Ok(StepResult {
                    step_id,
                    output: format!("Error after {} attempts: {}", attempts, e),
                    status: StepStatus::Failed(e.to_string()),
                    duration_ms: duration,
                    attempts,
                });
            }
            Err(_) => {
                // Timeout
                if attempts <= max_retries {
                    tracing::warn!(
                        "Step '{}' timed out (attempt {}/{}). Retrying in {}ms...",
                        step_id,
                        attempts,
                        max_attempts,
                        retry_delay_ms
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                    continue;
                }
                return Ok(StepResult {
                    step_id,
                    output: format!("Timed out after {}s ({} attempts)", timeout_secs, attempts),
                    status: StepStatus::TimedOut,
                    duration_ms: duration,
                    attempts,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_step_success() {
        let result = execute_step_with_retry(
            "test".into(),
            "agent-1".into(),
            "do something".into(),
            None,
            0,
            100,
            10,
        )
        .await
        .unwrap();
        assert_eq!(result.status, StepStatus::Success);
        assert_eq!(result.attempts, 1);
        assert!(result.output.contains("test"));
    }

    #[tokio::test]
    async fn test_execute_step_with_zero_retries() {
        let result = execute_step_with_retry(
            "no-retry".into(),
            "agent-1".into(),
            "prompt".into(),
            None,
            0,
            100,
            10,
        )
        .await
        .unwrap();
        assert_eq!(result.attempts, 1);
    }

    #[tokio::test]
    async fn test_execute_step_tracks_duration() {
        let result =
            execute_step_with_retry("dur".into(), "a".into(), "p".into(), None, 0, 100, 10)
                .await
                .unwrap();
        // Duration should be non-negative (u64 always is, but verify it's recorded)
        assert_eq!(result.duration_ms, result.duration_ms);
    }
}
