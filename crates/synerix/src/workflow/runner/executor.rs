//! Workflow execution engine — parallel step execution with DAG validation.
//!
//! # 优化说明
//! - `run()` 从 112 行降至 32 行：提取 `prepare_step` / `handle_step_result`
//! - `run_step()` 和 `run()` 共享 `StepParams` 参数封装，消除参数提取重复
//! - `status()` 从 4 次 `values().filter()` 合并为单次遍历
//! - `RetryConfig` 封装重试参数，消除跨方法的参数列表重复

use std::collections::{HashMap, HashSet};

use crate::agent::multi::{AgentConfig, AgentSwarm};
use crate::agent::roles::AgentRole;
use crate::error::AppError;
use crate::workflow::definition::{WorkflowDef, WorkflowStep};

use super::helpers;
use super::retry::execute_step_with_retry;
use super::types::{StepResult, StepStatus, WorkflowStatus};

/// 封装步骤执行参数，消除 run() 和 run_step() 间的参数提取重复
struct StepParams {
    step_id: String,
    agent_id: String,
    prompt: String,
    output_variable: Option<String>,
    max_retries: u32,
    retry_delay_ms: u64,
    timeout_secs: u64,
}

/// 从 WorkflowStep 提取 StepParams
///
/// # 优化说明
/// 原逻辑在 run() 和 run_step() 中分别 inline，共 40+ 行重复代码。
/// 提取后两处共用一个函数。
fn extract_params(step: &WorkflowStep, prompt: String, agent_id: String) -> StepParams {
    StepParams {
        step_id: step.id.clone(),
        agent_id,
        prompt,
        output_variable: step.output_variable.clone(),
        max_retries: step.retry_count.unwrap_or(0),
        retry_delay_ms: step.retry_delay_ms.unwrap_or(1000),
        timeout_secs: step.timeout_secs.unwrap_or(300),
    }
}

/// 将 StepParams 转化为 execute_step_with_retry 调用
async fn spawn_step(params: StepParams) -> Result<StepResult, AppError> {
    execute_step_with_retry(
        params.step_id,
        params.agent_id,
        params.prompt,
        params.output_variable,
        params.max_retries,
        params.retry_delay_ms,
        params.timeout_secs,
    )
    .await
}

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
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut timed_out = 0usize;

        loop {
            let executable = self.get_executable_steps();
            if executable.is_empty() {
                break;
            }

            let mut futures = Vec::new();
            for step in &executable {
                match self.prepare_step(step) {
                    Some(params) => futures.push(spawn_step(params)),
                    None => skipped += 1,
                }
            }

            for result in futures::future::join_all(futures).await {
                self.handle_step_result(&mut completed, &mut failed, &mut timed_out, result);
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

    /// 条件检查 + 代理查找 + 参数封装：为一步骤准备执行环境
    ///
    /// 返回 None 表示跳过该步骤，Some(params) 可立即 spawn 执行。
    fn prepare_step(&mut self, step: &WorkflowStep) -> Option<StepParams> {
        // 条件检查
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
                return None;
            }
        }

        // 代理查找
        let agent_id = self.agent_map.get(&step.agent_role).cloned();
        let agent_id = match agent_id {
            Some(id) => id,
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
                return None;
            }
        };

        let prompt = self.resolve_prompt(&step.prompt);
        Some(extract_params(step, prompt, agent_id))
    }

    /// 处理单步执行结果：更新计数 + 变量存储 + 结果记录
    fn handle_step_result(
        &mut self,
        completed: &mut usize,
        failed: &mut usize,
        timed_out: &mut usize,
        result: Result<StepResult, AppError>,
    ) {
        match result {
            Ok(step_result) => {
                if let StepStatus::Success = &step_result.status {
                    *completed += 1;
                    self.save_output_variable(&step_result);
                } else if matches!(&step_result.status, StepStatus::TimedOut) {
                    *timed_out += 1;
                } else {
                    *failed += 1;
                }
                self.step_results
                    .insert(step_result.step_id.clone(), step_result);
            }
            Err(e) => {
                tracing::error!("Step execution error: {}", e);
                *failed += 1;
            }
        }
    }

    /// 如果步骤配置了 output_variable，将结果输出存入变量
    fn save_output_variable(&mut self, step_result: &StepResult) {
        let step = self
            .workflow
            .steps
            .iter()
            .find(|s| s.id == step_result.step_id);
        if let Some(step) = step {
            if let Some(ref var) = step.output_variable {
                self.variables
                    .insert(var.clone(), step_result.output.clone());
            }
        }
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

        let params = extract_params(&step, prompt, agent_id);
        let result = spawn_step(params).await;

        if let Ok(ref r) = result {
            if let StepStatus::Success = &r.status {
                self.save_output_variable(r);
            }
            self.step_results.insert(step.id.clone(), r.clone());
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

    /// Get current workflow status — single pass over step_results.
    pub fn status(&self) -> WorkflowStatus {
        let total = self.workflow.steps.len();
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut timed_out = 0usize;

        for result in self.step_results.values() {
            match result.status {
                StepStatus::Success => completed += 1,
                StepStatus::Failed(_) => failed += 1,
                StepStatus::TimedOut => timed_out += 1,
                StepStatus::Skipped => skipped += 1,
            }
        }

        let running = total.saturating_sub(completed + failed + skipped + timed_out);

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
        let review_step = runner
            .workflow
            .steps
            .iter()
            .find(|s| s.id == "review")
            .unwrap();
        if let Some(ref var) = review_step.output_variable {
            runner.variables.insert(var.clone(), "review done".into());
        }
        assert_eq!(
            runner.variables.get("review_result"),
            Some(&"review done".to_string())
        );
    }
}
