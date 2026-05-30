//! Workflow execution engine — parallel step execution with DAG validation

mod executor;
mod helpers;
mod retry;
mod types;

pub use executor::WorkflowRunner;
pub use helpers::{evaluate_condition, get_executable_steps, resolve_prompt};
pub use types::{StepResult, StepStatus, WorkflowStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::multi::AgentSwarm;
    use crate::workflow::definition::{parse_workflow, WorkflowDef};

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
