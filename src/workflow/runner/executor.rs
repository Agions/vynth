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
