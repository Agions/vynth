//! Built-in workflow templates

use crate::workflow::definition::{WorkflowDef, WorkflowStep};
use std::collections::HashMap;

const DEFAULT_RETRY_DELAY_MS: u64 = 1000;

struct StepSpec {
    id: &'static str,
    agent_role: &'static str,
    prompt: &'static str,
    depends_on: &'static [&'static str],
    condition: Option<&'static str>,
    output_variable: Option<&'static str>,
    timeout_secs: u64,
}

fn workflow(name: &str, description: &str, steps: &[StepSpec]) -> WorkflowDef {
    WorkflowDef {
        name: name.to_string(),
        description: description.to_string(),
        version: "1.0".to_string(),
        steps: steps.iter().map(workflow_step).collect(),
        variables: HashMap::new(),
    }
}

fn workflow_step(spec: &StepSpec) -> WorkflowStep {
    WorkflowStep {
        id: spec.id.to_string(),
        agent_role: spec.agent_role.to_string(),
        prompt: spec.prompt.to_string(),
        depends_on: spec
            .depends_on
            .iter()
            .map(|dep| (*dep).to_string())
            .collect(),
        condition: spec.condition.map(str::to_string),
        output_variable: spec.output_variable.map(str::to_string),
        timeout_secs: Some(spec.timeout_secs),
        retry_count: None,
        retry_delay_ms: Some(DEFAULT_RETRY_DELAY_MS),
    }
}

/// Code review workflow: Coder → Reviewer → Tester
pub fn code_review_workflow() -> WorkflowDef {
    workflow(
        "code-review",
        "Automated code review pipeline: write, review, and test",
        &[
            StepSpec {
                id: "implement",
                agent_role: "coder",
                prompt: "{{task}}",
                depends_on: &[],
                condition: None,
                output_variable: Some("code_changes"),
                timeout_secs: 300,
            },
            StepSpec {
                id: "review",
                agent_role: "reviewer",
                prompt: "Review the following code changes and provide feedback:\n\n{{code_changes}}",
                depends_on: &["implement"],
                condition: Some("code_changes"),
                output_variable: Some("review_feedback"),
                timeout_secs: 120,
            },
            StepSpec {
                id: "test",
                agent_role: "tester",
                prompt: "Write tests for the following code:\n\n{{code_changes}}\n\nReview feedback to address:\n{{review_feedback}}",
                depends_on: &["review"],
                condition: Some("review_feedback"),
                output_variable: Some("test_results"),
                timeout_secs: 180,
            },
        ],
    )
}

/// Refactor workflow: Architect plans → Coder implements → Reviewer validates
pub fn refactor_workflow() -> WorkflowDef {
    workflow(
        "refactor",
        "Structured refactoring: plan, implement, validate",
        &[
            StepSpec {
                id: "plan",
                agent_role: "architect",
                prompt: "Create a refactoring plan for: {{task}}\n\nConsider: code structure, dependencies, risks, and migration steps.",
                depends_on: &[],
                condition: None,
                output_variable: Some("refactor_plan"),
                timeout_secs: 120,
            },
            StepSpec {
                id: "implement",
                agent_role: "coder",
                prompt: "Implement the following refactoring plan:\n\n{{refactor_plan}}",
                depends_on: &["plan"],
                condition: Some("refactor_plan"),
                output_variable: Some("refactored_code"),
                timeout_secs: 300,
            },
            StepSpec {
                id: "validate",
                agent_role: "reviewer",
                prompt: "Validate the refactoring:\n\nOriginal plan: {{refactor_plan}}\n\nRefactored code: {{refactored_code}}\n\nCheck: does the implementation match the plan? Any issues?",
                depends_on: &["implement"],
                condition: Some("refactored_code"),
                output_variable: Some("validation_result"),
                timeout_secs: 120,
            },
        ],
    )
}

/// Debug workflow: Tester reproduces → Coder fixes → Tester verifies
pub fn debug_workflow() -> WorkflowDef {
    workflow(
        "debug",
        "Systematic debugging: reproduce, fix, verify",
        &[
            StepSpec {
                id: "reproduce",
                agent_role: "tester",
                prompt: "Analyze and reproduce the following bug:\n\n{{task}}\n\nProvide: steps to reproduce, expected vs actual behavior, root cause analysis.",
                depends_on: &[],
                condition: None,
                output_variable: Some("bug_analysis"),
                timeout_secs: 120,
            },
            StepSpec {
                id: "fix",
                agent_role: "coder",
                prompt: "Fix the following bug:\n\n{{bug_analysis}}\n\nProvide the minimal code fix.",
                depends_on: &["reproduce"],
                condition: Some("bug_analysis"),
                output_variable: Some("bug_fix"),
                timeout_secs: 300,
            },
            StepSpec {
                id: "verify",
                agent_role: "tester",
                prompt: "Verify the bug fix:\n\nOriginal bug: {{bug_analysis}}\n\nApplied fix: {{bug_fix}}\n\nRun tests and confirm the fix works.",
                depends_on: &["fix"],
                condition: Some("bug_fix"),
                output_variable: Some("verification_result"),
                timeout_secs: 180,
            },
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_workflow() {
        let wf = code_review_workflow();
        assert_eq!(wf.name, "code-review");
        assert_eq!(wf.steps.len(), 3);
        assert_eq!(wf.steps[0].agent_role, "coder");
        assert_eq!(wf.steps[1].agent_role, "reviewer");
        assert_eq!(wf.steps[2].agent_role, "tester");
    }

    #[test]
    fn test_refactor_workflow() {
        let wf = refactor_workflow();
        assert_eq!(wf.name, "refactor");
        assert_eq!(wf.steps.len(), 3);
        assert_eq!(wf.steps[0].agent_role, "architect");
    }

    #[test]
    fn test_debug_workflow() {
        let wf = debug_workflow();
        assert_eq!(wf.name, "debug");
        assert_eq!(wf.steps.len(), 3);
        // Same agent role for reproduce and verify
        assert_eq!(wf.steps[0].agent_role, "tester");
        assert_eq!(wf.steps[2].agent_role, "tester");
    }

    #[test]
    fn test_workflow_dependencies() {
        let wf = code_review_workflow();
        assert!(wf.steps[0].depends_on.is_empty());
        assert_eq!(wf.steps[1].depends_on, vec!["implement"]);
        assert_eq!(wf.steps[2].depends_on, vec!["review"]);
    }
}
