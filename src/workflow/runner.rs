//! Workflow execution engine

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Success,
    Failed(String),
    Skipped,
}

/// Overall workflow status
#[derive(Debug, Clone)]
pub struct WorkflowStatus {
    pub total_steps: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub running: usize,
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
    pub fn new(workflow: WorkflowDef, mut swarm: AgentSwarm) -> Self {
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

        let mut variables = workflow.variables.clone();
        // Merge with runtime variables (runtime overrides default)
        for (k, v) in &workflow.variables {
            variables.entry(k.clone()).or_insert_with(|| v.clone());
        }

        Self {
            workflow,
            swarm,
            variables,
            step_results: HashMap::new(),
            agent_map,
        }
    }

    /// Run the entire workflow
    pub async fn run(&mut self) -> Result<WorkflowStatus, AppError> {
        let total = self.workflow.steps.len();
        let mut completed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Topological execution order
        loop {
            let executable = self.get_executable_steps();
            if executable.is_empty() {
                break;
            }

            for step in executable {
                // Check condition
                if let Some(ref cond) = step.condition {
                    if !self.evaluate_condition(cond) {
                        self.step_results.insert(
                            step.id.clone(),
                            StepResult {
                                step_id: step.id.clone(),
                                output: String::new(),
                                status: StepStatus::Skipped,
                                duration_ms: 0,
                            },
                        );
                        skipped += 1;
                        continue;
                    }
                }

                match self.run_step(&step.id).await {
                    Ok(result) => {
                        if result.status == StepStatus::Success {
                            completed += 1;
                        } else {
                            failed += 1;
                        }
                    }
                    Err(_) => failed += 1,
                }
            }
        }

        Ok(WorkflowStatus {
            total_steps: total,
            completed,
            failed,
            skipped,
            running: 0,
        })
    }

    /// Execute a single step
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

        let start = Instant::now();
        let result = self.swarm.run_task(&agent_id, &prompt).await;
        let duration = start.elapsed().as_millis() as u64;

        let step_result = match result {
            Ok(output) => {
                // Store output in variable if configured
                if let Some(ref var) = step.output_variable {
                    self.variables.insert(var.clone(), output.clone());
                }
                StepResult {
                    step_id: step.id.clone(),
                    output,
                    status: StepStatus::Success,
                    duration_ms: duration,
                }
            }
            Err(e) => StepResult {
                step_id: step.id.clone(),
                output: format!("Error: {}", e),
                status: StepStatus::Failed(e.to_string()),
                duration_ms: duration,
            },
        };

        self.step_results
            .insert(step.id.clone(), step_result.clone());
        Ok(step_result)
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

    /// Evaluate a simple condition (variable existence check)
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        let trimmed = condition.trim();
        // "var_name" -> true if variable exists and is non-empty
        if !trimmed.contains(' ') {
            return self
                .variables
                .get(trimmed)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
        }
        // "var_name != ''" -> true if variable exists
        if let Some(var_name) = trimmed.strip_suffix(" != ''") {
            return self.variables.contains_key(var_name.trim());
        }
        // "var_name == 'value'" -> exact match
        if let Some(pos) = trimmed.find(" == ") {
            let var_name = trimmed[..pos].trim();
            let expected = trimmed[pos + 4..].trim().trim_matches('\'');
            return self
                .variables
                .get(var_name)
                .map(|v| v == expected)
                .unwrap_or(false);
        }
        true
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
        let running = total - completed - failed - skipped;

        WorkflowStatus {
            total_steps: total,
            completed,
            failed,
            skipped,
            running,
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

    #[test]
    fn test_runner_creation() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm);
        assert_eq!(runner.agent_map.len(), 2); // coder + reviewer
        assert_eq!(runner.variables.get("language").unwrap(), "Rust");
    }

    #[test]
    fn test_resolve_prompt() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm);
        let resolved = runner.resolve_prompt("Write hello {{language}}");
        assert_eq!(resolved, "Write hello Rust");
    }

    #[test]
    fn test_evaluate_condition_exists() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm);
        assert!(!runner.evaluate_condition("code_output"));
        runner.variables.insert("code_output".into(), "done".into());
        assert!(runner.evaluate_condition("code_output"));
    }

    #[test]
    fn test_evaluate_condition_empty() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm);
        runner.variables.insert("code_output".into(), "".into());
        assert!(!runner.evaluate_condition("code_output"));
    }

    #[test]
    fn test_get_executable_steps_initial() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let runner = WorkflowRunner::new(wf, swarm);
        let steps = runner.get_executable_steps();
        assert_eq!(steps.len(), 1); // only "code" (review depends on code)
        assert_eq!(steps[0].id, "code");
    }

    #[test]
    fn test_get_executable_steps_after_completion() {
        let wf = sample_workflow();
        let swarm = AgentSwarm::new();
        let mut runner = WorkflowRunner::new(wf, swarm);
        runner.step_results.insert(
            "code".into(),
            StepResult {
                step_id: "code".into(),
                output: "hello".into(),
                status: StepStatus::Success,
                duration_ms: 100,
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
        let mut runner = WorkflowRunner::new(wf, swarm);
        let status = runner.status();
        assert_eq!(status.total_steps, 2);
        assert_eq!(status.completed, 0);
        assert_eq!(status.running, 2);
    }
}
