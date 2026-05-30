use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::agent::multi::{AgentConfig, AgentSwarm};
use crate::agent::roles::AgentRole;
use crate::error::AppError;
use crate::workflow::definition::{WorkflowDef, WorkflowStep};

use super::types::{StepResult, StepStatus, WorkflowStatus};

/// Workflow execution engine
pub struct WorkflowRunner {
    pub workflow: WorkflowDef,
    pub swarm: AgentSwarm,
    pub variables: HashMap<String, String>,
    pub step_results: HashMap<String, StepResult>,
    pub(crate) agent_map: HashMap<String, String>, // role -> agent_id
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
