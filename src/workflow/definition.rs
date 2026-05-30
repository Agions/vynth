//! Workflow definition types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    /// Workflow name
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Version string
    #[serde(default = "default_version")]
    pub version: String,
    /// Ordered list of steps
    pub steps: Vec<WorkflowStep>,
    /// Default variables (can be overridden at runtime)
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique step identifier
    pub id: String,
    /// Which agent role executes this step
    pub agent_role: String,
    /// Prompt template (supports {{variable}} interpolation)
    pub prompt: String,
    /// Step IDs that must complete before this step
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional condition expression
    #[serde(default)]
    pub condition: Option<String>,
    /// Store result in this variable
    #[serde(default)]
    pub output_variable: Option<String>,
    /// Timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parse workflow from YAML
pub fn parse_workflow(yaml: &str) -> Result<WorkflowDef, crate::error::AppError> {
    serde_yaml::from_str(yaml)
        .map_err(|e| crate::error::AppError::Config(format!("Invalid workflow YAML: {}", e)))
}

/// Parse workflow from TOML
pub fn parse_workflow_toml(toml_str: &str) -> Result<WorkflowDef, crate::error::AppError> {
    toml::from_str(toml_str)
        .map_err(|e| crate::error::AppError::Config(format!("Invalid workflow TOML: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflow_yaml() {
        let yaml = r#"
name: test-workflow
description: A test workflow
version: "1.0"
steps:
  - id: step1
    agent_role: coder
    prompt: "Write hello world"
  - id: step2
    agent_role: reviewer
    prompt: "Review the code"
    depends_on:
      - step1
"#;
        let wf = parse_workflow(yaml).unwrap();
        assert_eq!(wf.name, "test-workflow");
        assert_eq!(wf.steps.len(), 2);
        assert_eq!(wf.steps[0].id, "step1");
        assert_eq!(wf.steps[1].depends_on, vec!["step1"]);
    }

    #[test]
    fn test_parse_workflow_toml() {
        let toml_str = r#"
name = "test-workflow"
description = "A test workflow"

[[steps]]
id = "step1"
agent_role = "coder"
prompt = "Write hello world"

[[steps]]
id = "step2"
agent_role = "reviewer"
prompt = "Review the code"
depends_on = ["step1"]
"#;
        let wf = parse_workflow_toml(toml_str).unwrap();
        assert_eq!(wf.name, "test-workflow");
        assert_eq!(wf.steps.len(), 2);
    }

    #[test]
    fn test_workflow_step_defaults() {
        let yaml = r#"
name: minimal
steps:
  - id: s1
    agent_role: coder
    prompt: "do something"
"#;
        let wf = parse_workflow(yaml).unwrap();
        assert!(wf.steps[0].depends_on.is_empty());
        assert!(wf.steps[0].condition.is_none());
        assert!(wf.steps[0].output_variable.is_none());
    }

    #[test]
    fn test_parse_invalid_workflow() {
        let result = parse_workflow("not valid yaml: [[[[");
        assert!(result.is_err());
    }
}
