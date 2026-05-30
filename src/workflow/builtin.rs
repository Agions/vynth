//! Built-in workflow templates

use crate::workflow::definition::{WorkflowDef, WorkflowStep};
use std::collections::HashMap;

/// Code review workflow: Coder → Reviewer → Tester
pub fn code_review_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "code-review".to_string(),
        description: "Automated code review pipeline: write, review, and test".to_string(),
        version: "1.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "implement".to_string(),
                agent_role: "coder".to_string(),
                prompt: "{{task}}".to_string(),
                depends_on: vec![],
                condition: None,
                output_variable: Some("code_changes".to_string()),
                timeout_secs: Some(300),
            },
            WorkflowStep {
                id: "review".to_string(),
                agent_role: "reviewer".to_string(),
                prompt: "Review the following code changes and provide feedback:\n\n{{code_changes}}".to_string(),
                depends_on: vec!["implement".to_string()],
                condition: Some("code_changes".to_string()),
                output_variable: Some("review_feedback".to_string()),
                timeout_secs: Some(120),
            },
            WorkflowStep {
                id: "test".to_string(),
                agent_role: "tester".to_string(),
                prompt: "Write tests for the following code:\n\n{{code_changes}}\n\nReview feedback to address:\n{{review_feedback}}".to_string(),
                depends_on: vec!["review".to_string()],
                condition: Some("review_feedback".to_string()),
                output_variable: Some("test_results".to_string()),
                timeout_secs: Some(180),
            },
        ],
        variables: HashMap::new(),
    }
}

/// Refactor workflow: Architect plans → Coder implements → Reviewer validates
pub fn refactor_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "refactor".to_string(),
        description: "Structured refactoring: plan, implement, validate".to_string(),
        version: "1.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "plan".to_string(),
                agent_role: "architect".to_string(),
                prompt: "Create a refactoring plan for: {{task}}\n\nConsider: code structure, dependencies, risks, and migration steps.".to_string(),
                depends_on: vec![],
                condition: None,
                output_variable: Some("refactor_plan".to_string()),
                timeout_secs: Some(120),
            },
            WorkflowStep {
                id: "implement".to_string(),
                agent_role: "coder".to_string(),
                prompt: "Implement the following refactoring plan:\n\n{{refactor_plan}}".to_string(),
                depends_on: vec!["plan".to_string()],
                condition: Some("refactor_plan".to_string()),
                output_variable: Some("refactored_code".to_string()),
                timeout_secs: Some(300),
            },
            WorkflowStep {
                id: "validate".to_string(),
                agent_role: "reviewer".to_string(),
                prompt: "Validate the refactoring:\n\nOriginal plan: {{refactor_plan}}\n\nRefactored code: {{refactored_code}}\n\nCheck: does the implementation match the plan? Any issues?".to_string(),
                depends_on: vec!["implement".to_string()],
                condition: Some("refactored_code".to_string()),
                output_variable: Some("validation_result".to_string()),
                timeout_secs: Some(120),
            },
        ],
        variables: HashMap::new(),
    }
}

/// Debug workflow: Tester reproduces → Coder fixes → Tester verifies
pub fn debug_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "debug".to_string(),
        description: "Systematic debugging: reproduce, fix, verify".to_string(),
        version: "1.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "reproduce".to_string(),
                agent_role: "tester".to_string(),
                prompt: "Analyze and reproduce the following bug:\n\n{{task}}\n\nProvide: steps to reproduce, expected vs actual behavior, root cause analysis.".to_string(),
                depends_on: vec![],
                condition: None,
                output_variable: Some("bug_analysis".to_string()),
                timeout_secs: Some(120),
            },
            WorkflowStep {
                id: "fix".to_string(),
                agent_role: "coder".to_string(),
                prompt: "Fix the following bug:\n\n{{bug_analysis}}\n\nProvide the minimal code fix.".to_string(),
                depends_on: vec!["reproduce".to_string()],
                condition: Some("bug_analysis".to_string()),
                output_variable: Some("bug_fix".to_string()),
                timeout_secs: Some(300),
            },
            WorkflowStep {
                id: "verify".to_string(),
                agent_role: "tester".to_string(),
                prompt: "Verify the bug fix:\n\nOriginal bug: {{bug_analysis}}\n\nApplied fix: {{bug_fix}}\n\nRun tests and confirm the fix works.".to_string(),
                depends_on: vec!["fix".to_string()],
                condition: Some("bug_fix".to_string()),
                output_variable: Some("verification_result".to_string()),
                timeout_secs: Some(180),
            },
        ],
        variables: HashMap::new(),
    }
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
