//! Workflow engine — YAML-defined multi-step agent pipelines

pub mod builtin;
pub mod definition;
pub mod runner;

pub use builtin::{code_review_workflow, debug_workflow, refactor_workflow};
pub use definition::{parse_workflow, parse_workflow_toml, WorkflowDef, WorkflowStep};
pub use runner::{StepResult, StepStatus, WorkflowRunner, WorkflowStatus};
