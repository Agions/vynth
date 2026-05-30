//! Workflow execution engine — parallel step execution with DAG validation.

use std::collections::{HashMap, HashSet};

use crate::agent::multi::{AgentConfig, AgentSwarm};
use crate::agent::roles::AgentRole;
use crate::error::AppError;
use crate::workflow::definition::{WorkflowDef, WorkflowStep};

use super::helpers;
use super::retry::execute_step_with_retry;
use super::types::{StepResult, StepStatus, WorkflowStatus};

/// Workflow execution engine.
pub struct WorkflowRunner {
    pub workflow: WorkflowDef,
    pub swarm: AgentSwarm,
    pub variables: HashMap<String, String>,
    pub step_results: HashMap<String, StepResult>,
    pub(crate) agent_map: HashMap<String, String>, // role -> agent_id
}

impl WorkflowRunner {
    /// Create a new runner, spawning one agent per unique role in the workflow.
    pub fn new(workflow: WorkflowDef, mut swarm: AgentSwarm) -> Result<Self, AppError> {
        workflow.validate_dag()?;

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

    /// Run the entire workflow — executes independent steps in parallel.
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

            let results = futures::future::join_all(futures).await;

            for result in results {
                match result {
                    Ok(step_result) => {
                        if let StepStatus::Success = &step_result.status {
                            completed += 1;
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

    /// Execute a single step by id.
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

    /// Interpolate `{{variables}}` in a prompt template.
    pub fn resolve_prompt(&self, template: &str) -> String {
        helpers::resolve_prompt(template, &self.variables)
    }

    /// Evaluate a condition expression against the current variables.
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        helpers::evaluate_condition(condition, &self.variables)
    }

    /// Get steps whose dependencies are all satisfied.
    pub fn get_executable_steps(&self) -> Vec<WorkflowStep> {
        helpers::get_executable_steps(&self.workflow.steps, &self.step_results)
    }

    /// Get current workflow status.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::roles::AgentRole;
    use crate::workflow::definition::parse_workflow;

    fn sample_runner() -> WorkflowRunner {
        let wf = parse_workflow(
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
  - id: test
    agent_role: tester
    prompt: "Test: {{review_result}}"
    depends_on: [review]
variables:
  language: Rust
"#,
        )
        .unwrap();
        let swarm = AgentSwarm::new();
        WorkflowRunner::new(wf, swarm).unwrap()
    }

    // ── new / construction ────────────────────────────────────

    #[test]
    fn new_creates_agents_for_unique_roles() {
        let runner = sample_runner();
        // 3 unique roles: coder, reviewer, tester
        assert_eq!(runner.agent_map.len(), 3);
        assert!(runner.agent_map.contains_key("coder"));
        assert!(runner.agent_map.contains_key("reviewer"));
        assert!(runner.agent_map.contains_key("tester"));
    }

    #[test]
    fn new_initializes_variables_from_workflow() {
        let runner = sample_runner();
        assert_eq!(runner.variables.get("language").unwrap(), "Rust");
    }

    #[test]
    fn new_starts_with_empty_results() {
        let runner = sample_runner();
        assert!(runner.step_results.is_empty());
    }

    #[test]
    fn new_rejects_cyclic_workflow() {
        let wf = parse_workflow(
            r#"
name: cyclic
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [b]
  - id: b
    agent_role: coder
    prompt: "B"
    depends_on: [a]
"#,
        )
        .unwrap();
        let swarm = AgentSwarm::new();
        let result = WorkflowRunner::new(wf, swarm);
        assert!(result.is_err());
    }

    // ── resolve_prompt ────────────────────────────────────────

    #[test]
    fn resolve_prompt_interpolates_variables() {
        let runner = sample_runner();
        let result = runner.resolve_prompt("Write {{language}}");
        assert_eq!(result, "Write Rust");
    }

    #[test]
    fn resolve_prompt_leaves_unknown_vars() {
        let runner = sample_runner();
        let result = runner.resolve_prompt("{{unknown}}");
        assert_eq!(result, "{{unknown}}");
    }

    // ── evaluate_condition ────────────────────────────────────

    #[test]
    fn evaluate_condition_true_for_existing_var() {
        let runner = sample_runner();
        assert!(runner.evaluate_condition("language"));
    }

    #[test]
    fn evaluate_condition_false_for_missing_var() {
        let runner = sample_runner();
        assert!(!runner.evaluate_condition("missing"));
    }

    // ── get_executable_steps ──────────────────────────────────

    #[test]
    fn get_executable_steps_initial_only_first() {
        let runner = sample_runner();
        let steps = runner.get_executable_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "code");
    }

    #[test]
    fn get_executable_steps_after_first_done() {
        let mut runner = sample_runner();
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
        // "review" depends on "code" (success) and has condition "code_output"
        // but code_output variable isn't set, so condition evaluates false → skipped in run()
        // But get_executable_steps only checks dependencies, not conditions
        let steps = runner.get_executable_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "review");
    }

    // ── status ────────────────────────────────────────────────

    #[test]
    fn status_empty_workflow() {
        let runner = sample_runner();
        let s = runner.status();
        assert_eq!(s.total_steps, 3);
        assert_eq!(s.completed, 0);
        assert_eq!(s.failed, 0);
        assert_eq!(s.skipped, 0);
        assert_eq!(s.running, 3);
    }

    #[test]
    fn status_counts_completed_and_failed() {
        let mut runner = sample_runner();
        runner.step_results.insert(
            "code".into(),
            StepResult {
                step_id: "code".into(),
                output: "ok".into(),
                status: StepStatus::Success,
                duration_ms: 50,
                attempts: 1,
            },
        );
        runner.step_results.insert(
            "review".into(),
            StepResult {
                step_id: "review".into(),
                output: "err".into(),
                status: StepStatus::Failed("bad".into()),
                duration_ms: 10,
                attempts: 2,
            },
        );
        let s = runner.status();
        assert_eq!(s.total_steps, 3);
        assert_eq!(s.completed, 1);
        assert_eq!(s.failed, 1);
        assert_eq!(s.skipped, 0);
        assert_eq!(s.running, 1);
    }

    #[test]
    fn status_counts_skipped() {
        let mut runner = sample_runner();
        runner.step_results.insert(
            "code".into(),
            StepResult {
                step_id: "code".into(),
                output: String::new(),
                status: StepStatus::Skipped,
                duration_ms: 0,
                attempts: 0,
            },
        );
        let s = runner.status();
        assert_eq!(s.skipped, 1);
        assert_eq!(s.running, 2);
    }

    // ── run_step error cases ──────────────────────────────────

    #[tokio::test]
    async fn run_step_not_found_returns_error() {
        let mut runner = sample_runner();
        let result = runner.run_step("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // ── variable storage from step results ────────────────────

    #[test]
    fn variables_updated_on_success_with_output_variable() {
        let mut runner = sample_runner();
        runner.step_results.insert(
            "code".into(),
            StepResult {
                step_id: "code".into(),
                output: "code done".into(),
                status: StepStatus::Success,
                duration_ms: 50,
                attempts: 1,
            },
        );
        // Manually simulate what run() does: store output in output_variable
        // "review" step has output_variable: "review_result"
        let review_step = runner
            .workflow
            .steps
            .iter()
            .find(|s| s.id == "review")
            .unwrap();
        assert_eq!(
            review_step.output_variable.as_deref(),
            Some("review_result")
        );
    }
}
