//! Workflow execution engine — parallel step execution with DAG validation

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::agent::multi::{AgentConfig, AgentSwarm};
use crate::agent::roles::AgentRole;
use crate::error::AppError;
use crate::workflow::definition::{WorkflowDef, WorkflowStep};

/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub output: String,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Success,
    Failed(String),
    Skipped,
    TimedOut,
}

/// Overall workflow status
#[derive(Debug, Clone)]
pub struct WorkflowStatus {
    pub total_steps: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub running: usize,
    pub timed_out: usize,
}

/// Workflow execution engine
pub struct WorkflowRunner {
    pub workflow: WorkflowDef,
    pub swarm: AgentSwarm,
    pub variables: HashMap<String, String>,
    pub step_results: HashMap<String, StepResult>,
    agent_map: HashMap<String, String>, // role -> agent_id
}

impl WorkflowRunner {
    pub fn new(workflow: WorkflowDef, mut swarm: AgentSwarm) -> Result<Self, AppError> {
        // Validate DAG before proceeding
        workflow.validate_dag()?;

        // Create agents for each unique role in the workflow
        let mut agent_map = HashMap::new();
        let mut seen_roles = HashSet::new();

        for step in &workflow.steps {
            if seen_roles.insert(step.agent_role.clone()) {
                let role = match step.agent_role.as_str() {
                    "coder" => AgentRole::Coder,
                    "reviewer" => AgentRole::Reviewer,
                    "tester" => AgentRole::Tester,
                    "architect" => AgentRole::Architect,
                    "planner" => AgentRole::Planner,
                    other => AgentRole::Custom(other.to_string()),
                };
                let config = AgentConfig::new(role, &step.agent_role);
                let agent_id = swarm.spawn_agent(config);
                agent_map.insert(step.agent_role.clone(), agent_id);
            }
        }

        let variables = workflow.variables.clone();

        Ok(Self {
            workflow,
            swarm,
            variables,
            step_results: HashMap::new(),
            agent_map,
        })
    }

    /// Run the entire workflow — executes independent steps in parallel
    pub async fn run(&mut self) -> Result<WorkflowStatus, AppError> {
        let total = self.workflow.steps.len();
        let mut completed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut timed_out = 0;

        loop {
            let executable = self.get_executable_steps();
            if executable.is_empty() {
                break;
            }

            // Execute all independent steps in parallel
            let mut futures = Vec::new();
            for step in &executable {
                // Check condition before spawning
                if let Some(ref cond) = step.condition {
                    if !self.evaluate_condition(cond) {
                        self.step_results.insert(
                            step.id.clone(),
                            StepResult {
                                step_id: step.id.clone(),
                                output: String::new(),
                                status: StepStatus::Skipped,
                                duration_ms: 0,
                                attempts: 0,
                            },
                        );
                        skipped += 1;
                        continue;
                    }
                }

                let prompt = self.resolve_prompt(&step.prompt);
                let agent_id = match self.agent_map.get(&step.agent_role) {
                    Some(id) => id.clone(),
                    None => {
                        self.step_results.insert(
                            step.id.clone(),
                            StepResult {
                                step_id: step.id.clone(),
                                output: format!("No agent for role '{}'", step.agent_role),
                                status: StepStatus::Failed(format!(
                                    "No agent for role '{}'",
                                    step.agent_role
                                )),
                                duration_ms: 0,
                                attempts: 0,
                            },
                        );
                        failed += 1;
                        continue;
                    }
                };

                let max_retries = step.retry_count.unwrap_or(0);
                let retry_delay = step.retry_delay_ms.unwrap_or(1000);
                let timeout_secs = step.timeout_secs.unwrap_or(300);

                futures.push(execute_step_with_retry(
                    step.id.clone(),
                    agent_id,
                    prompt,
                    step.output_variable.clone(),
                    max_retries,
                    retry_delay,
                    timeout_secs,
                ));
            }

            // Await all parallel steps
            let results = futures::future::join_all(futures).await;

            for result in results {
                match result {
                    Ok(step_result) => {
                        // Store output in variable if configured
                        if let StepStatus::Success = &step_result.status {
                            completed += 1;
                            // Find the step to get output_variable
                            if let Some(step) = self
                                .workflow
                                .steps
                                .iter()
                                .find(|s| s.id == step_result.step_id)
                            {
                                if let Some(ref var) = step.output_variable {
                                    self.variables
                                        .insert(var.clone(), step_result.output.clone());
                                }
                            }
                        } else if matches!(&step_result.status, StepStatus::TimedOut) {
                            timed_out += 1;
                        } else {
                            failed += 1;
                        }
                        self.step_results
                            .insert(step_result.step_id.clone(), step_result);
                    }
                    Err(e) => {
                        tracing::error!("Step execution error: {}", e);
                        failed += 1;
                    }
                }
            }
        }

        Ok(WorkflowStatus {
            total_steps: total,
            completed,
            failed,
            skipped,
            running: 0,
            timed_out,
        })
    }

    /// Execute a single step (wrapper for backward compatibility)
    pub async fn run_step(&mut self, step_id: &str) -> Result<StepResult, AppError> {
        let step = self
            .workflow
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| AppError::ExecutionFailed(format!("Step '{}' not found", step_id)))?
            .clone();

        let prompt = self.resolve_prompt(&step.prompt);
        let agent_id = self
            .agent_map
            .get(&step.agent_role)
            .ok_or_else(|| {
                AppError::ExecutionFailed(format!("No agent for role '{}'", step.agent_role))
            })?
            .clone();

        let max_retries = step.retry_count.unwrap_or(0);
        let retry_delay = step.retry_delay_ms.unwrap_or(1000);
        let timeout_secs = step.timeout_secs.unwrap_or(300);

        let result = execute_step_with_retry(
            step.id.clone(),
            agent_id,
            prompt,
            step.output_variable.clone(),
            max_retries,
            retry_delay,
            timeout_secs,
        )
        .await;

        match &result {
            Ok(r) => {
                if let StepStatus::Success = &r.status {
                    if let Some(ref var) = step.output_variable {
                        self.variables.insert(var.clone(), r.output.clone());
                    }
                }
                self.step_results.insert(step.id.clone(), r.clone());
            }
            Err(_) => {}
        }

        result
    }

    /// Interpolate {{variables}} in a prompt template
    pub fn resolve_prompt(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Evaluate a condition expression. Supports:
    /// - `var_name` — true if variable exists and is non-empty
    /// - `!var_name` — true if variable is missing or empty
    /// - `var_name != 'value'` — not equal
    /// - `var_name == 'value'` — exact match
    /// - `var_name contains 'value'` — substring match
    /// - `var_name starts_with 'value'` — prefix match
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        let trimmed = condition.trim();

        // Negation: !var_name
        if let Some(var_name) = trimmed.strip_prefix('!') {
            return self
                .variables
                .get(var_name.trim())
                .map(|v| v.is_empty())
                .unwrap_or(true);
        }

        // No spaces → variable existence check
        if !trimmed.contains(' ') {
            return self
                .variables
                .get(trimmed)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
        }

        // var_name != 'value'
        if let Some(rest) = trimmed.strip_suffix('\'') {
            if let Some(pos) = trimmed.find(" != '") {
                let var_name = trimmed[..pos].trim();
                let expected = &trimmed[pos + 5..rest.len()];
                return self
                    .variables
                    .get(var_name)
                    .map(|v| v != expected)
                    .unwrap_or(true);
            }
        }

        // var_name == 'value'
        if let Some(pos) = trimmed.find(" == '") {
            let var_name = trimmed[..pos].trim();
            let expected = trimmed[pos + 5..].trim().trim_matches('\'');
            return self
                .variables
                .get(var_name)
                .map(|v| v == expected)
                .unwrap_or(false);
        }

        // var_name contains 'value'
        if let Some(pos) = trimmed.find(" contains '") {
            let var_name = trimmed[..pos].trim();
            let needle = trimmed[pos + 11..].trim().trim_matches('\'');
            return self
                .variables
                .get(var_name)
                .map(|v| v.contains(needle))
                .unwrap_or(false);
        }

        // var_name starts_with 'value'
        if let Some(pos) = trimmed.find(" starts_with '") {
            let var_name = trimmed[..pos].trim();
            let prefix = trimmed[pos + 14..].trim().trim_matches('\'');
            return self
                .variables
                .get(var_name)
                .map(|v| v.starts_with(prefix))
                .unwrap_or(false);
        }

        // Fallback: treat as variable existence
        self.variables
            .get(trimmed)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// Get steps whose dependencies are all satisfied
    pub fn get_executable_steps(&self) -> Vec<WorkflowStep> {
        self.workflow
            .steps
            .iter()
            .filter(|step| {
                // Not already executed
                !self.step_results.contains_key(&step.id)
                // All dependencies satisfied
                && step
                    .depends_on
                    .iter()
                    .all(|dep| self.step_results.get(dep).map(|r| r.status == StepStatus::Success).unwrap_or(false))
            })
            .cloned()
            .collect()
    }

    /// Get current workflow status
    pub fn status(&self) -> WorkflowStatus {
        let total = self.workflow.steps.len();
        let completed = self
            .step_results
            .values()
            .filter(|r| r.status == StepStatus::Success)
            .count();
        let failed = self
            .step_results
            .values()
            .filter(|r| matches!(r.status, StepStatus::Failed(_)))
            .count();
        let skipped = self
            .step_results
            .values()
            .filter(|r| r.status == StepStatus::Skipped)
            .count();
        let timed_out = self
            .step_results
            .values()
            .filter(|r| r.status == StepStatus::TimedOut)
            .count();
        let running = total - completed - failed - skipped - timed_out;

        WorkflowStatus {
            total_steps: total,
            completed,
            failed,
            skipped,
            running,
            timed_out,
        }
    }
}

/// Execute a step with retry logic and timeout.
/// This is a free function to avoid borrowing issues with the swarm.
async fn execute_step_with_retry(
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
            Ok::<String, AppError>(format!(
                "[Step '{}' completed] (simulated)",
                step_id
            ))
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
                    output: format!(
                        "Timed out after {}s ({} attempts)",
                        timeout_secs, attempts
                    ),
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
    use crate::workflow::definition::parse_workflow;

    fn sample_workflow() -> WorkflowDef {
        parse_workflow(
            r#"
name: test-wf
steps:
  - id: code
    agent_role: coder
    prompt: "Write hello {{language}}"
  - id: review
    agent_role: reviewer
    prompt: "Review: {{code_output}}"
    depends_on: [code]
    condition: code_output
    output_variable: review_result
variables:
  language: Rust
"#,
        )
        .unwrap()
    }

    fn workflow_with_retry() -> WorkflowDef {
        parse_workflow(
            r#"
name: retry-wf
steps:
  - id: step1
    agent_role: coder
    prompt: "Do something"
    retry_count: 2
    retry_delay_ms: 100
    timeout_secs: 5
  - id: step2
    agent_role: reviewer
    prompt: "Review {{step1}}"
    depends_on: [step1]
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_runner_creation() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm).unwrap();
        assert_eq!(runner.agent_map.len(), 2); // coder + reviewer
        assert_eq!(runner.variables.get("language").unwrap(), "Rust");
    }

    #[test]
    fn test_resolve_prompt() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm).unwrap();
        let resolved = runner.resolve_prompt("Write hello {{language}}");
        assert_eq!(resolved, "Write hello Rust");
    }

    #[test]
    fn test_evaluate_condition_exists() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        assert!(!runner.evaluate_condition("code_output"));
        runner.variables.insert("code_output".into(), "done".into());
        assert!(runner.evaluate_condition("code_output"));
    }

    #[test]
    fn test_evaluate_condition_empty() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        runner.variables.insert("code_output".into(), "".into());
        assert!(!runner.evaluate_condition("code_output"));
    }

    #[test]
    fn test_evaluate_condition_negation() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        assert!(runner.evaluate_condition("!code_output"));
        runner.variables.insert("code_output".into(), "done".into());
        assert!(!runner.evaluate_condition("!code_output"));
    }

    #[test]
    fn test_evaluate_condition_not_equal() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        runner.variables.insert("status".into(), "error".into());
        assert!(runner.evaluate_condition("status != 'ok'"));
        assert!(!runner.evaluate_condition("status != 'error'"));
    }

    #[test]
    fn test_evaluate_condition_contains() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        runner
            .variables
            .insert("output".into(), "hello world".into());
        assert!(runner.evaluate_condition("output contains 'world'"));
        assert!(!runner.evaluate_condition("output contains 'rust'"));
    }

    #[test]
    fn test_evaluate_condition_starts_with() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        runner
            .variables
            .insert("path".into(), "/usr/local/bin".into());
        assert!(runner.evaluate_condition("path starts_with '/usr'"));
        assert!(!runner.evaluate_condition("path starts_with '/home'"));
    }

    #[test]
    fn test_get_executable_steps_initial() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm).unwrap();
        let steps = runner.get_executable_steps();
        assert_eq!(steps.len(), 1); // only "code" (review depends on code)
        assert_eq!(steps[0].id, "code");
    }

    #[test]
    fn test_get_executable_steps_after_completion() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm).unwrap();
        runner.step_results.insert(
            "code".into(),
            StepResult {
                step_id: "code".into(),
                output: "hello".into(),
                status: StepStatus::Success,
                duration_ms: 100,
                attempts: 1,
            },
        );
        let steps = runner.get_executable_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "review");
    }

    #[test]
    fn test_workflow_status() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm).unwrap();
        let status = runner.status();
        assert_eq!(status.total_steps, 2);
        assert_eq!(status.completed, 0);
        assert_eq!(status.running, 2);
    }

    #[test]
    fn test_dag_validation_success() {
        let wf = sample_workflow();
        assert!(wf.validate_dag().is_ok());
    }

    #[test]
    fn test_dag_validation_cycle() {
        let wf = parse_workflow(
            r#"
name: cyclic-wf
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [b]
  - id: b
    agent_role: reviewer
    prompt: "B"
    depends_on: [a]
"#,
        )
        .unwrap();
        assert!(wf.validate_dag().is_err());
    }

    #[test]
    fn test_workflow_with_retry_fields() {
        let wf = workflow_with_retry();
        assert_eq!(wf.steps[0].retry_count, Some(2));
        assert_eq!(wf.steps[0].retry_delay_ms, Some(100));
        assert_eq!(wf.steps[0].timeout_secs, Some(5));
    }

    #[test]
    fn test_runner_dag_validation_on_creation() {
        let wf = parse_workflow(
            r#"
name: bad-wf
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [a]
"#,
        )
        .unwrap();
        let swarm = AgentSwarm::new();
        let result = WorkflowRunner::new(wf, swarm);
        assert!(result.is_err());
    }
}
