//! Workflow helper functions — prompt interpolation, condition evaluation,
//! and dependency resolution.

use std::collections::HashMap;

use crate::workflow::definition::WorkflowStep;

use super::types::StepStatus;

/// Interpolate `{{variables}}` in a prompt template.
pub fn resolve_prompt(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
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
pub fn evaluate_condition(condition: &str, variables: &HashMap<String, String>) -> bool {
    let trimmed = condition.trim();

    // Negation: !var_name
    if let Some(var_name) = trimmed.strip_prefix('!') {
        return variables
            .get(var_name.trim())
            .map(|v| v.is_empty())
            .unwrap_or(true);
    }

    // No spaces → variable existence check
    if !trimmed.contains(' ') {
        return variables
            .get(trimmed)
            .map(|v| !v.is_empty())
            .unwrap_or(false);
    }

    // var_name != 'value'
    if let Some(rest) = trimmed.strip_suffix('\'') {
        if let Some(pos) = trimmed.find(" != '") {
            let var_name = trimmed[..pos].trim();
            let expected = &trimmed[pos + 5..rest.len()];
            return variables
                .get(var_name)
                .map(|v| v != expected)
                .unwrap_or(true);
        }
    }

    // var_name == 'value'
    if let Some(pos) = trimmed.find(" == '") {
        let var_name = trimmed[..pos].trim();
        let expected = trimmed[pos + 5..].trim().trim_matches('\'');
        return variables
            .get(var_name)
            .map(|v| v == expected)
            .unwrap_or(false);
    }

    // var_name contains 'value'
    if let Some(pos) = trimmed.find(" contains '") {
        let var_name = trimmed[..pos].trim();
        let needle = trimmed[pos + 11..].trim().trim_matches('\'');
        return variables
            .get(var_name)
            .map(|v| v.contains(needle))
            .unwrap_or(false);
    }

    // var_name starts_with 'value'
    if let Some(pos) = trimmed.find(" starts_with '") {
        let var_name = trimmed[..pos].trim();
        let prefix = trimmed[pos + 14..].trim().trim_matches('\'');
        return variables
            .get(var_name)
            .map(|v| v.starts_with(prefix))
            .unwrap_or(false);
    }

    // Fallback: treat as variable existence
    variables
        .get(trimmed)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Get steps whose dependencies are all satisfied.
pub fn get_executable_steps(
    steps: &[WorkflowStep],
    step_results: &HashMap<String, super::types::StepResult>,
) -> Vec<WorkflowStep> {
    steps
        .iter()
        .filter(|step| {
            // Not already executed
            !step_results.contains_key(&step.id)
            // All dependencies satisfied
            && step
                .depends_on
                .iter()
                .all(|dep| step_results.get(dep).map(|r| r.status == StepStatus::Success).unwrap_or(false))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::multi::AgentSwarm;
    use crate::workflow::definition::parse_workflow;
    use crate::workflow::runner::types::StepResult;
    use crate::workflow::runner::WorkflowRunner;

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
variables:
  language: Rust
"#,
        )
        .unwrap();
        let swarm = AgentSwarm::new();
        WorkflowRunner::new(wf, swarm).unwrap()
    }

    #[test]
    fn test_resolve_prompt_basic() {
        let mut vars = HashMap::new();
        vars.insert("language".into(), "Rust".into());
        assert_eq!(resolve_prompt("Write {{language}}", &vars), "Write Rust");
    }

    #[test]
    fn test_resolve_prompt_missing_var() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_prompt("Write {{missing}}", &vars),
            "Write {{missing}}"
        );
    }

    #[test]
    fn test_evaluate_condition_exists() {
        let mut vars = HashMap::new();
        assert!(!evaluate_condition("code_output", &vars));
        vars.insert("code_output".into(), "done".into());
        assert!(evaluate_condition("code_output", &vars));
    }

    #[test]
    fn test_evaluate_condition_empty() {
        let mut vars = HashMap::new();
        vars.insert("code_output".into(), "".into());
        assert!(!evaluate_condition("code_output", &vars));
    }

    #[test]
    fn test_evaluate_condition_negation() {
        let mut vars = HashMap::new();
        assert!(evaluate_condition("!code_output", &vars));
        vars.insert("code_output".into(), "done".into());
        assert!(!evaluate_condition("!code_output", &vars));
    }

    #[test]
    fn test_evaluate_condition_not_equal() {
        let mut vars = HashMap::new();
        vars.insert("status".into(), "error".into());
        assert!(evaluate_condition("status != 'ok'", &vars));
        assert!(!evaluate_condition("status != 'error'", &vars));
    }

    #[test]
    fn test_evaluate_condition_contains() {
        let mut vars = HashMap::new();
        vars.insert("output".into(), "hello world".into());
        assert!(evaluate_condition("output contains 'world'", &vars));
        assert!(!evaluate_condition("output contains 'rust'", &vars));
    }

    #[test]
    fn test_evaluate_condition_starts_with() {
        let mut vars = HashMap::new();
        vars.insert("path".into(), "/usr/local/bin".into());
        assert!(evaluate_condition("path starts_with '/usr'", &vars));
        assert!(!evaluate_condition("path starts_with '/home'", &vars));
    }

    #[test]
    fn test_get_executable_steps_initial() {
        let runner = sample_runner();
        let steps = get_executable_steps(&runner.workflow.steps, &runner.step_results);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "code");
    }

    #[test]
    fn test_get_executable_steps_after_completion() {
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
        let steps = get_executable_steps(&runner.workflow.steps, &runner.step_results);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "review");
    }
}
