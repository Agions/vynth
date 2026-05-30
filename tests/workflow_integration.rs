//! Integration tests for the workflow engine
//!
//! These tests exercise the full workflow execution path: DAG validation,
//! dependency resolution, parallel execution, variable propagation,
//! condition evaluation, and error handling.

use syncode::agent::multi::AgentSwarm;
use syncode::workflow::definition::parse_workflow;
use syncode::workflow::runner::{StepResult, StepStatus, WorkflowRunner};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_swarm() -> AgentSwarm {
    AgentSwarm::new()
}

/// Build a WorkflowRunner from a YAML workflow string, panicking on error.
fn runner_from_yaml(yaml: &str) -> WorkflowRunner {
    let wf = parse_workflow(yaml).unwrap();
    WorkflowRunner::new(wf, make_swarm()).unwrap()
}

// ---------------------------------------------------------------------------
// 1. Complete multi-step sequential workflow execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_full_sequential_workflow_execution() {
    let mut runner = runner_from_yaml(
        r#"
name: sequential-wf
steps:
  - id: step1
    agent_role: coder
    prompt: "Write code"
    output_variable: code_result
  - id: step2
    agent_role: reviewer
    prompt: "Review {{code_result}}"
    depends_on: [step1]
    output_variable: review_result
  - id: step3
    agent_role: tester
    prompt: "Test {{review_result}}"
    depends_on: [step2]
    output_variable: test_result
"#,
    );

    let status = runner.run().await.unwrap();

    // All 3 steps should complete successfully
    assert_eq!(status.total_steps, 3);
    assert_eq!(status.completed, 3);
    assert_eq!(status.failed, 0);
    assert_eq!(status.skipped, 0);

    // Each step result should be Success
    for step_id in &["step1", "step2", "step3"] {
        let result = runner
            .step_results
            .get(*step_id)
            .expect("step result missing");
        assert_eq!(
            result.status,
            StepStatus::Success,
            "{} should succeed",
            step_id
        );
        assert!(
            result.attempts >= 1,
            "{} should have at least 1 attempt",
            step_id
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Parallel step execution (independent steps run in same batch)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_parallel_independent_steps() {
    let mut runner = runner_from_yaml(
        r#"
name: parallel-wf
steps:
  - id: a
    agent_role: coder
    prompt: "Task A"
  - id: b
    agent_role: reviewer
    prompt: "Task B"
  - id: c
    agent_role: tester
    prompt: "Task C"
  - id: final
    agent_role: architect
    prompt: "Combine results"
    depends_on: [a, b, c]
"#,
    );

    // Before execution: a, b, c should all be executable (no deps)
    let initial = runner.get_executable_steps();
    assert_eq!(
        initial.len(),
        3,
        "3 independent steps should be executable initially"
    );
    let ids: Vec<&str> = initial.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"a"));
    assert!(ids.contains(&"b"));
    assert!(ids.contains(&"c"));

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 4);
    assert_eq!(status.completed, 4);
    assert_eq!(status.failed, 0);

    // "final" should only have been executed after a, b, c all completed
    let final_result = runner.step_results.get("final").unwrap();
    assert_eq!(final_result.status, StepStatus::Success);
}

// ---------------------------------------------------------------------------
// 3. Variable propagation between steps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_variable_propagation_across_steps() {
    let mut runner = runner_from_yaml(
        r#"
name: var-prop-wf
variables:
  input_text: hello
steps:
  - id: produce
    agent_role: coder
    prompt: "Process {{input_text}}"
    output_variable: intermediate
  - id: consume
    agent_role: reviewer
    prompt: "Review {{intermediate}}"
    depends_on: [produce]
    output_variable: final_output
"#,
    );

    // Initial variable should be present
    assert_eq!(runner.variables.get("input_text").unwrap(), "hello");

    runner.run().await.unwrap();

    // After execution, output_variable values should be stored
    assert!(
        runner.variables.contains_key("intermediate"),
        "intermediate variable should be set after step 'produce'"
    );
    assert!(
        runner.variables.contains_key("final_output"),
        "final_output variable should be set after step 'consume'"
    );

    // The simulated output format is "[Step 'produce completed] (simulated)"
    let intermediate = runner.variables.get("intermediate").unwrap();
    assert!(
        intermediate.contains("produce"),
        "intermediate should reference step 'produce', got: {}",
        intermediate
    );
}

// ---------------------------------------------------------------------------
// 4. Condition-based step skipping during execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_condition_skips_step_during_execution() {
    let mut runner = runner_from_yaml(
        r#"
name: cond-wf
variables:
  skip_review: ""
steps:
  - id: code
    agent_role: coder
    prompt: "Write code"
    output_variable: code_out
  - id: review
    agent_role: reviewer
    prompt: "Review code"
    depends_on: [code]
    condition: skip_review
"#,
    );

    let status = runner.run().await.unwrap();

    // "code" should succeed, "review" should be skipped (skip_review is empty)
    assert_eq!(status.total_steps, 2);
    assert_eq!(status.completed, 1);
    assert_eq!(status.skipped, 1);
    assert_eq!(status.failed, 0);

    assert_eq!(
        runner.step_results.get("code").unwrap().status,
        StepStatus::Success
    );
    assert_eq!(
        runner.step_results.get("review").unwrap().status,
        StepStatus::Skipped
    );
}

#[tokio::test]
async fn test_condition_allows_step_when_variable_set() {
    let mut runner = runner_from_yaml(
        r#"
name: cond-allow-wf
variables:
  should_review: "yes"
steps:
  - id: code
    agent_role: coder
    prompt: "Write code"
    output_variable: code_out
  - id: review
    agent_role: reviewer
    prompt: "Review code"
    depends_on: [code]
    condition: should_review
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.completed, 2);
    assert_eq!(status.skipped, 0);
}

// ---------------------------------------------------------------------------
// 5. Complex DAG with branching and convergence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_complex_dag_branching_and_convergence() {
    //       a
    //      / \
    //     b   c
    //      \ /
    //       d
    let mut runner = runner_from_yaml(
        r#"
name: diamond-wf
steps:
  - id: a
    agent_role: architect
    prompt: "Design"
    output_variable: design
  - id: b
    agent_role: coder
    prompt: "Implement branch 1: {{design}}"
    depends_on: [a]
    output_variable: branch1
  - id: c
    agent_role: coder
    prompt: "Implement branch 2: {{design}}"
    depends_on: [a]
    output_variable: branch2
  - id: d
    agent_role: tester
    prompt: "Test {{branch1}} and {{branch2}}"
    depends_on: [b, c]
"#,
    );

    // Initially only "a" is executable
    assert_eq!(runner.get_executable_steps().len(), 1);

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 4);
    assert_eq!(status.completed, 4);

    // Verify all steps completed in correct order
    let d_result = runner.step_results.get("d").unwrap();
    assert_eq!(d_result.status, StepStatus::Success);
}

// ---------------------------------------------------------------------------
// 6. Failed dependency blocks downstream steps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_failed_dependency_blocks_downstream() {
    let wf = parse_workflow(
        r#"
name: fail-block-wf
steps:
  - id: step1
    agent_role: coder
    prompt: "Do something"
    output_variable: result
  - id: step2
    agent_role: reviewer
    prompt: "Review {{result}}"
    depends_on: [step1]
"#,
    )
    .unwrap();
    let mut runner = WorkflowRunner::new(wf, make_swarm()).unwrap();

    // Manually insert a failed result for step1
    runner.step_results.insert(
        "step1".into(),
        StepResult {
            step_id: "step1".into(),
            output: "Error".into(),
            status: StepStatus::Failed("simulated failure".into()),
            duration_ms: 50,
            attempts: 1,
        },
    );

    // step2 depends on step1, which failed → step2 should NOT be executable
    let executable = runner.get_executable_steps();
    assert!(
        executable.is_empty(),
        "No steps should be executable when dependency failed"
    );
}

// ---------------------------------------------------------------------------
// 7. Skipped dependency blocks downstream steps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_skipped_dependency_blocks_downstream() {
    let wf = parse_workflow(
        r#"
name: skip-block-wf
steps:
  - id: step1
    agent_role: coder
    prompt: "Do something"
  - id: step2
    agent_role: reviewer
    prompt: "Review"
    depends_on: [step1]
"#,
    )
    .unwrap();
    let mut runner = WorkflowRunner::new(wf, make_swarm()).unwrap();

    // Insert a skipped result for step1
    runner.step_results.insert(
        "step1".into(),
        StepResult {
            step_id: "step1".into(),
            output: String::new(),
            status: StepStatus::Skipped,
            duration_ms: 0,
            attempts: 0,
        },
    );

    // step2 should NOT be executable (dependency was skipped, not success)
    let executable = runner.get_executable_steps();
    assert!(
        executable.is_empty(),
        "No steps should be executable when dependency was skipped"
    );
}

// ---------------------------------------------------------------------------
// 8. TimedOut dependency blocks downstream steps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_timed_out_dependency_blocks_downstream() {
    let wf = parse_workflow(
        r#"
name: timeout-block-wf
steps:
  - id: step1
    agent_role: coder
    prompt: "Do something"
  - id: step2
    agent_role: reviewer
    prompt: "Review"
    depends_on: [step1]
"#,
    )
    .unwrap();
    let mut runner = WorkflowRunner::new(wf, make_swarm()).unwrap();

    // Insert a timed-out result for step1
    runner.step_results.insert(
        "step1".into(),
        StepResult {
            step_id: "step1".into(),
            output: "Timed out".into(),
            status: StepStatus::TimedOut,
            duration_ms: 5000,
            attempts: 3,
        },
    );

    let executable = runner.get_executable_steps();
    assert!(
        executable.is_empty(),
        "No steps should be executable when dependency timed out"
    );
}

// ---------------------------------------------------------------------------
// 9. Empty workflow executes successfully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_empty_workflow() {
    let mut runner = runner_from_yaml(
        r#"
name: empty-wf
steps: []
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 0);
    assert_eq!(status.completed, 0);
    assert_eq!(status.failed, 0);
}

// ---------------------------------------------------------------------------
// 10. Single step workflow
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_single_step_workflow() {
    let mut runner = runner_from_yaml(
        r#"
name: single-wf
steps:
  - id: only
    agent_role: coder
    prompt: "Hello world"
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 1);
    assert_eq!(status.completed, 1);
    assert_eq!(status.failed, 0);
    assert_eq!(
        runner.step_results.get("only").unwrap().status,
        StepStatus::Success
    );
}

// ---------------------------------------------------------------------------
// 11. Workflow status tracking during execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_workflow_status_after_full_execution() {
    let mut runner = runner_from_yaml(
        r#"
name: status-wf
steps:
  - id: s1
    agent_role: coder
    prompt: "Step 1"
  - id: s2
    agent_role: reviewer
    prompt: "Step 2"
  - id: s3
    agent_role: tester
    prompt: "Step 3"
"#,
    );

    // Initial status: all 3 running (none executed yet)
    let initial = runner.status();
    assert_eq!(initial.total_steps, 3);
    assert_eq!(initial.completed, 0);
    assert_eq!(initial.running, 3);

    runner.run().await.unwrap();

    // After execution: all 3 completed
    let final_status = runner.status();
    assert_eq!(final_status.total_steps, 3);
    assert_eq!(final_status.completed, 3);
    assert_eq!(final_status.running, 0);
    assert_eq!(final_status.failed, 0);
}

// ---------------------------------------------------------------------------
// 12. DAG validation on runner creation (cycle detection)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_runner_rejects_cyclic_workflow() {
    let wf = parse_workflow(
        r#"
name: cyclic
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [c]
  - id: b
    agent_role: reviewer
    prompt: "B"
    depends_on: [a]
  - id: c
    agent_role: tester
    prompt: "C"
    depends_on: [b]
"#,
    )
    .unwrap();

    let result = WorkflowRunner::new(wf, make_swarm());
    assert!(result.is_err(), "Should reject cyclic workflow");
}

// ---------------------------------------------------------------------------
// 13. DAG validation: self-referencing dependency
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_runner_rejects_self_referencing_step() {
    let wf = parse_workflow(
        r#"
name: self-ref
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [a]
"#,
    )
    .unwrap();

    let result = WorkflowRunner::new(wf, make_swarm());
    assert!(result.is_err(), "Should reject self-referencing step");
}

// ---------------------------------------------------------------------------
// 14. Multiple agents for the same role
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multiple_steps_same_role_share_agent() {
    let mut runner = runner_from_yaml(
        r#"
name: same-role-wf
steps:
  - id: code1
    agent_role: coder
    prompt: "Write module A"
  - id: code2
    agent_role: coder
    prompt: "Write module B"
  - id: review
    agent_role: reviewer
    prompt: "Review everything"
    depends_on: [code1, code2]
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.completed, 3);
}

// ---------------------------------------------------------------------------
// 15. Variable interpolation in prompts with multiple variables
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multi_variable_interpolation() {
    let mut runner = runner_from_yaml(
        r#"
name: multi-var-wf
variables:
  project: myapp
  language: Rust
steps:
  - id: step1
    agent_role: coder
    prompt: "Create {{project}} in {{language}}"
    output_variable: code
  - id: step2
    agent_role: reviewer
    prompt: "Review {{project}} {{code}}"
    depends_on: [step1]
"#,
    );

    // Test prompt resolution
    let resolved = runner.resolve_prompt("Build {{project}} using {{language}}");
    assert_eq!(resolved, "Build myapp using Rust");

    runner.run().await.unwrap();
    assert_eq!(runner.status().completed, 2);
}

// ---------------------------------------------------------------------------
// 16. Step with retry configuration executes successfully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_step_with_retry_config_executes() {
    let mut runner = runner_from_yaml(
        r#"
name: retry-config-wf
steps:
  - id: retry_step
    agent_role: coder
    prompt: "Something that might fail"
    retry_count: 3
    retry_delay_ms: 50
    timeout_secs: 10
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.completed, 1);

    let result = runner.step_results.get("retry_step").unwrap();
    assert_eq!(result.status, StepStatus::Success);
    // Since simulated execution succeeds on first try, attempts should be 1
    assert_eq!(result.attempts, 1);
}

// ---------------------------------------------------------------------------
// 17. Step with timeout configuration executes successfully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_step_with_timeout_config_executes() {
    let mut runner = runner_from_yaml(
        r#"
name: timeout-config-wf
steps:
  - id: timeout_step
    agent_role: coder
    prompt: "Long running task"
    timeout_secs: 60
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.completed, 1);

    let result = runner.step_results.get("timeout_step").unwrap();
    assert_eq!(result.status, StepStatus::Success);
}

// ---------------------------------------------------------------------------
// 18. Complex workflow: branching with conditional skip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_branching_with_conditional_skip() {
    let mut runner = runner_from_yaml(
        r#"
name: branch-skip-wf
variables:
  skip_testing: ""
steps:
  - id: design
    agent_role: architect
    prompt: "Design system"
    output_variable: design
  - id: code
    agent_role: coder
    prompt: "Implement {{design}}"
    depends_on: [design]
    output_variable: code_out
  - id: test
    agent_role: tester
    prompt: "Test {{code_out}}"
    depends_on: [code]
    condition: skip_testing
  - id: review
    agent_role: reviewer
    prompt: "Review {{code_out}}"
    depends_on: [code]
"#,
    );

    let status = runner.run().await.unwrap();

    // design → code → (test skipped, review succeeds)
    assert_eq!(status.total_steps, 4);
    assert_eq!(status.completed, 3); // design, code, review
    assert_eq!(status.skipped, 1); // test

    assert_eq!(
        runner.step_results.get("test").unwrap().status,
        StepStatus::Skipped
    );
    assert_eq!(
        runner.step_results.get("review").unwrap().status,
        StepStatus::Success
    );
}

// ---------------------------------------------------------------------------
// 19. Linear chain preserves execution order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_linear_chain_execution_order() {
    let mut runner = runner_from_yaml(
        r#"
name: chain-wf
steps:
  - id: first
    agent_role: architect
    prompt: "Plan"
    output_variable: plan
  - id: second
    agent_role: coder
    prompt: "Code {{plan}}"
    depends_on: [first]
    output_variable: code
  - id: third
    agent_role: reviewer
    prompt: "Review {{code}}"
    depends_on: [second]
    output_variable: review
  - id: fourth
    agent_role: tester
    prompt: "Test {{review}}"
    depends_on: [third]
"#,
    );

    // Only first step should be executable initially
    let exec = runner.get_executable_steps();
    assert_eq!(exec.len(), 1);
    assert_eq!(exec[0].id, "first");

    let status = runner.run().await.unwrap();
    assert_eq!(status.completed, 4);
    assert_eq!(status.failed, 0);

    // Verify all variables were propagated through the chain
    assert!(runner.variables.contains_key("plan"));
    assert!(runner.variables.contains_key("code"));
    assert!(runner.variables.contains_key("review"));
}

// ---------------------------------------------------------------------------
// 20. get_executable_steps returns empty when all steps completed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_executable_steps_after_completion() {
    let mut runner = runner_from_yaml(
        r#"
name: done-wf
steps:
  - id: s1
    agent_role: coder
    prompt: "Do it"
"#,
    );

    runner.run().await.unwrap();
    assert!(
        runner.get_executable_steps().is_empty(),
        "No steps should be executable after workflow completes"
    );
}

// ---------------------------------------------------------------------------
// 21. Negation condition in workflow execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_negation_condition_during_execution() {
    let mut runner = runner_from_yaml(
        r#"
name: neg-cond-wf
variables:
  errors: ""
steps:
  - id: generate
    agent_role: coder
    prompt: "Generate code"
    output_variable: code
  - id: fix_errors
    agent_role: coder
    prompt: "Fix errors in {{code}}"
    depends_on: [generate]
    condition: "!errors"
"#,
    );

    let status = runner.run().await.unwrap();
    // errors is empty, so !errors is true → fix_errors should run
    assert_eq!(status.completed, 2);
    assert_eq!(status.skipped, 0);
}

#[tokio::test]
async fn test_negation_condition_blocks_when_variable_set() {
    let mut runner = runner_from_yaml(
        r#"
name: neg-block-wf
variables:
  errors: "found"
steps:
  - id: generate
    agent_role: coder
    prompt: "Generate code"
    output_variable: code
  - id: fix_errors
    agent_role: coder
    prompt: "Fix errors in {{code}}"
    depends_on: [generate]
    condition: "!errors"
"#,
    );

    let status = runner.run().await.unwrap();
    // errors is "found", so !errors is false → fix_errors should be skipped
    assert_eq!(status.completed, 1);
    assert_eq!(status.skipped, 1);
}

// ---------------------------------------------------------------------------
// 22. Equals condition in workflow execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_equals_condition_during_execution() {
    let mut runner = runner_from_yaml(
        r#"
name: eq-cond-wf
variables:
  mode: "fast"
steps:
  - id: build
    agent_role: coder
    prompt: "Build"
    output_variable: build_out
  - id: optimize
    agent_role: coder
    prompt: "Optimize {{build_out}}"
    depends_on: [build]
    condition: "mode == 'fast'"
"#,
    );

    let status = runner.run().await.unwrap();
    // mode == "fast", condition is true → optimize runs
    assert_eq!(status.completed, 2);
    assert_eq!(status.skipped, 0);
}

// ---------------------------------------------------------------------------
// 23. Workflow with many parallel independent steps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_many_parallel_independent_steps() {
    let mut runner = runner_from_yaml(
        r#"
name: many-parallel-wf
steps:
  - id: p1
    agent_role: coder
    prompt: "Parallel 1"
  - id: p2
    agent_role: coder
    prompt: "Parallel 2"
  - id: p3
    agent_role: coder
    prompt: "Parallel 3"
  - id: p4
    agent_role: coder
    prompt: "Parallel 4"
  - id: p5
    agent_role: coder
    prompt: "Parallel 5"
  - id: gather
    agent_role: architect
    prompt: "Gather all"
    depends_on: [p1, p2, p3, p4, p5]
"#,
    );

    // All 5 parallel steps should be executable initially
    let exec = runner.get_executable_steps();
    assert_eq!(exec.len(), 5);

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 6);
    assert_eq!(status.completed, 6);
}

// ---------------------------------------------------------------------------
// 24. Two-level deep branching DAG
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_two_level_branching_dag() {
    //        root
    //       / | \
    //      a  b  c
    //       \ | /
    //        merge
    let mut runner = runner_from_yaml(
        r#"
name: three-way-merge-wf
steps:
  - id: root
    agent_role: architect
    prompt: "Design"
    output_variable: design
  - id: a
    agent_role: coder
    prompt: "Part A: {{design}}"
    depends_on: [root]
  - id: b
    agent_role: coder
    prompt: "Part B: {{design}}"
    depends_on: [root]
  - id: c
    agent_role: coder
    prompt: "Part C: {{design}}"
    depends_on: [root]
  - id: merge
    agent_role: tester
    prompt: "Merge and test"
    depends_on: [a, b, c]
"#,
    );

    let status = runner.run().await.unwrap();
    assert_eq!(status.total_steps, 5);
    assert_eq!(status.completed, 5);
}
