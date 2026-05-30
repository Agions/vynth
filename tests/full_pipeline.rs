//! Full pipeline integration tests
//!
//! Tests the complete flow: config → tool registry → skill registry →
//! session store → context manager → workflow parsing.

use std::path::Path;

// ── Config Loading from TOML ───────────────────────────────

#[test]
fn test_config_load_and_defaults() {
    let settings = synerix::config::Settings::load().unwrap();
    // Verify core fields
    assert!(!settings.llm.model.is_empty());
    assert!(settings.llm.context_window > 0);
    assert!(settings.llm.max_output_tokens > 0);
    assert!(!settings.ui.theme.is_empty());
    assert!(!settings.ui.keymap.is_empty());
}

#[test]
fn test_config_roundtrip_preserves_all_sections() {
    let settings = synerix::config::Settings::load().unwrap();
    let toml_str = toml::to_string_pretty(&settings).unwrap();
    let parsed: synerix::config::Settings = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.llm.model, settings.llm.model);
    assert_eq!(parsed.llm.context_window, settings.llm.context_window);
    assert_eq!(parsed.llm.temperature, settings.llm.temperature);
    assert_eq!(parsed.ui.theme, settings.ui.theme);
    assert_eq!(parsed.ui.keymap, settings.ui.keymap);
    assert_eq!(parsed.sandbox.mode, settings.sandbox.mode);
}

#[test]
fn test_config_from_custom_toml_string() {
    let toml = r#"
[llm]
provider = "deepseek"
api_key = "sk-test-pipeline"
model = "deepseek-reasoner"
context_window = 64000
max_output_tokens = 4096
temperature = 0.3

[ui]
theme = "light"
keymap = "vim"
typing_delay_ms = 20

[sandbox]
mode = "confirm"
atomic_writes = true

[[mcp]]
name = "test-server"
transport = { type = "stdio", command = "test-mcp", args = ["--port", "8080"] }
allowed_tools = ["tool_a", "tool_b"]

[[agents]]
name = "pipeline-test-agent"
system_prompt = "You are a pipeline tester"
max_turns = 3
tags = ["test", "pipeline"]
"#;
    let settings: synerix::config::Settings = toml::from_str(toml).unwrap();

    assert_eq!(settings.llm.model, "deepseek-reasoner");
    assert_eq!(settings.llm.context_window, 64000);
    assert_eq!(settings.llm.temperature, 0.3);
    assert_eq!(settings.ui.theme, "light");
    assert_eq!(settings.ui.keymap, "vim");
    assert_eq!(settings.sandbox.mode, synerix::config::SandboxMode::Confirm);
    assert_eq!(settings.mcp.len(), 1);
    assert_eq!(settings.mcp[0].name, "test-server");
    assert_eq!(settings.mcp[0].allowed_tools, vec!["tool_a", "tool_b"]);
    assert_eq!(settings.agents.len(), 1);
    assert_eq!(settings.agents[0].name, "pipeline-test-agent");
    assert_eq!(settings.agents[0].max_turns, 3);
    assert_eq!(settings.agents[0].tags, vec!["test", "pipeline"]);
}

// ── Tool Registry with Builtin Tools ───────────────────────

#[test]
fn test_tool_registry_full_initialization() {
    let mut registry = synerix::tools::ToolRegistry::new();
    synerix::tools::builtin::register_builtins(&mut registry);

    let names = registry.list_names();
    assert_eq!(names.len(), 5);

    // All builtin tools must be present
    for expected in &["file_read", "file_write", "shell_exec", "search", "patch"] {
        assert!(
            names.contains(expected),
            "Missing builtin tool: {}",
            expected
        );
    }
}

#[test]
fn test_tool_registry_schemas_are_complete() {
    let mut registry = synerix::tools::ToolRegistry::new();
    synerix::tools::builtin::register_builtins(&mut registry);

    let schemas = registry.all_schemas();
    assert_eq!(schemas.len(), 5);

    for schema in &schemas {
        assert!(!schema.function.name.is_empty());
        assert!(!schema.function.description.is_empty());
        assert_eq!(schema.schema_type, "function");
        // Parameters must be a valid JSON object
        assert!(schema.function.parameters.is_object());
    }
}

#[test]
fn test_tool_registry_schema_caching() {
    let mut registry = synerix::tools::ToolRegistry::new();
    synerix::tools::builtin::register_builtins(&mut registry);

    // First call builds cache
    let schemas1 = registry.all_schemas();
    assert!(registry.cached_schemas().is_some());

    // Second call returns cached
    let schemas2 = registry.all_schemas();
    assert_eq!(schemas1.len(), schemas2.len());

    // Verify cached schemas match
    for (s1, s2) in schemas1.iter().zip(schemas2.iter()) {
        assert_eq!(s1.function.name, s2.function.name);
    }
}

#[test]
fn test_tool_registry_individual_lookup() {
    let mut registry = synerix::tools::ToolRegistry::new();
    synerix::tools::builtin::register_builtins(&mut registry);

    // Each tool should be retrievable by name
    for name in &["file_read", "file_write", "shell_exec", "search", "patch"] {
        let tool = registry.get(name);
        assert!(tool.is_some(), "Tool '{}' not found", name);
        assert_eq!(tool.unwrap().name(), *name);
    }

    // Non-existent tool
    assert!(registry.get("nonexistent_tool").is_none());
}

#[test]
fn test_tool_registry_empty_before_registration() {
    let registry = synerix::tools::ToolRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.list_names().is_empty());
    assert!(registry.all_schemas().is_empty());
}

// ── Skill Registry ─────────────────────────────────────────

#[test]
fn test_skill_registry_from_builtin() {
    let code_review = synerix::skills::builtin::code_review_skill();
    assert_eq!(code_review.name, "code-review");
    assert!(!code_review.instructions.is_empty());

    let refactor = synerix::skills::builtin::refactor_skill();
    assert_eq!(refactor.name, "refactor");
    assert!(!refactor.instructions.is_empty());
}

#[test]
fn test_skill_registry_empty() {
    let registry = synerix::skills::SkillRegistry::new();
    assert!(registry.list_names().is_empty());
    assert!(registry.get("anything").is_none());
}

#[tokio::test]
async fn test_skill_registry_load_from_nonexistent_dir() {
    let registry = synerix::skills::SkillRegistry::load_from_dir(Path::new("/nonexistent/skills"))
        .await
        .unwrap();
    assert!(registry.list_names().is_empty());
}

#[tokio::test]
async fn test_skill_registry_load_from_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let registry = synerix::skills::SkillRegistry::load_from_dir(dir.path())
        .await
        .unwrap();
    assert!(registry.list_names().is_empty());
}

#[tokio::test]
async fn test_skill_registry_load_from_dir_with_skills() {
    let dir = tempfile::tempdir().unwrap();

    // Write a valid skill file
    let skill_content = r#"---
name: test-pipeline-skill
description: A skill for pipeline testing
instructions: ""
trigger:
  type: auto_match
  keywords: ["pipeline", "test"]
  threshold: 0.5
required_tools: ["file_read"]
---

# Pipeline Test Skill

When testing the pipeline, focus on integration between components.
"#;
    let skill_path = dir.path().join("pipeline-skill.md");
    std::fs::write(&skill_path, skill_content).unwrap();

    let registry = synerix::skills::SkillRegistry::load_from_dir(dir.path())
        .await
        .unwrap();

    let names = registry.list_names();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"test-pipeline-skill"));

    // Verify skill content
    let skill = registry.get("test-pipeline-skill").unwrap();
    assert_eq!(skill.description, "A skill for pipeline testing");
    assert!(skill.instructions.contains("Pipeline Test Skill"));
    assert_eq!(skill.required_tools, vec!["file_read"]);
}

#[tokio::test]
async fn test_skill_registry_matching_and_instructions() {
    let dir = tempfile::tempdir().unwrap();

    let skill1 = r#"---
name: code-helper
description: Helps with code
instructions: ""
trigger:
  type: auto_match
  keywords: ["code", "help", "debug"]
  threshold: 0.5
---
Help with code.
"#;
    let skill2 = r#"---
name: test-helper
description: Helps with tests
instructions: ""
trigger:
  type: auto_match
  keywords: ["test", "testing", "assert"]
  threshold: 0.5
---
Help with tests.
"#;
    std::fs::write(dir.path().join("code.md"), skill1).unwrap();
    std::fs::write(dir.path().join("test.md"), skill2).unwrap();

    let registry = synerix::skills::SkillRegistry::load_from_dir(dir.path())
        .await
        .unwrap();
    assert_eq!(registry.list_names().len(), 2);

    // Match with "code" keyword
    let matched = registry.match_skills("help me debug this code");
    assert!(!matched.is_empty());

    // Build instructions from matched skills
    let instructions = registry.build_instructions(&matched);
    assert!(instructions.contains("Active Skills"));
}

// ── Session Store CRUD Operations ──────────────────────────

#[test]
fn test_session_create_and_read() {
    let store = synerix::session::SessionStore::memory().unwrap();
    let session = synerix::session::Session::new("Pipeline Test Session", "test-model");

    store.create_session(&session).unwrap();

    let sessions = store.list_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, "Pipeline Test Session");
    assert_eq!(sessions[0].model, "test-model");
    assert_eq!(sessions[0].total_tokens, 0);
    assert_eq!(sessions[0].message_count, 0);
}

#[test]
fn test_session_update_via_messages() {
    let store = synerix::session::SessionStore::memory().unwrap();
    let session = synerix::session::Session::new("Update Test", "model");
    store.create_session(&session).unwrap();

    // Add messages to update session stats
    let mut msg1 = synerix::session::StoredMessage::user(&session.id, "First message");
    msg1.tokens_used = 50;
    store.save_message(&msg1).unwrap();

    let mut msg2 = synerix::session::StoredMessage::assistant(&session.id, "Response");
    msg2.tokens_used = 100;
    store.save_message(&msg2).unwrap();

    // Verify session stats updated
    let sessions = store.list_sessions().unwrap();
    assert_eq!(sessions[0].message_count, 2);
    assert_eq!(sessions[0].total_tokens, 150);
}

#[test]
fn test_session_delete_isolation() {
    let store = synerix::session::SessionStore::memory().unwrap();

    // Create two sessions
    let s1 = synerix::session::Session::new("Session 1", "m1");
    let s2 = synerix::session::Session::new("Session 2", "m2");
    store.create_session(&s1).unwrap();
    store.create_session(&s2).unwrap();

    // Add messages to both
    store
        .save_message(&synerix::session::StoredMessage::user(&s1.id, "msg in s1"))
        .unwrap();
    store
        .save_message(&synerix::session::StoredMessage::user(&s2.id, "msg in s2"))
        .unwrap();

    // Load messages for each — they should be isolated
    let msgs1 = store.load_messages(&s1.id).unwrap();
    let msgs2 = store.load_messages(&s2.id).unwrap();

    assert_eq!(msgs1.len(), 1);
    assert_eq!(msgs1[0].content, "msg in s1");
    assert_eq!(msgs2.len(), 1);
    assert_eq!(msgs2[0].content, "msg in s2");
}

#[test]
fn test_session_message_roles() {
    let store = synerix::session::SessionStore::memory().unwrap();
    let session = synerix::session::Session::new("Role Test", "model");
    store.create_session(&session).unwrap();

    // Save messages with different roles
    store
        .save_message(&synerix::session::StoredMessage::user(&session.id, "user msg"))
        .unwrap();
    store
        .save_message(&synerix::session::StoredMessage::assistant(&session.id, "asst msg"))
        .unwrap();

    let messages = store.load_messages(&session.id).unwrap();
    assert_eq!(messages.len(), 2);
    assert!(matches!(
        messages[0].role,
        synerix::session::StoredRole::User
    ));
    assert!(matches!(
        messages[1].role,
        synerix::session::StoredRole::Assistant
    ));
}

#[test]
fn test_session_message_ordering() {
    let store = synerix::session::SessionStore::memory().unwrap();
    let session = synerix::session::Session::new("Order Test", "model");
    store.create_session(&session).unwrap();

    // Messages are ordered by timestamp
    let msgs = vec!["first", "second", "third", "fourth"];
    for content in &msgs {
        store
            .save_message(&synerix::session::StoredMessage::user(
                &session.id,
                content,
            ))
            .unwrap();
    }

    let loaded = store.load_messages(&session.id).unwrap();
    assert_eq!(loaded.len(), 4);
    for (i, msg) in loaded.iter().enumerate() {
        assert_eq!(msg.content, msgs[i]);
    }
}

#[test]
fn test_session_multiple_sessions_list_order() {
    let store = synerix::session::SessionStore::memory().unwrap();

    store
        .create_session(&synerix::session::Session::new("First", "m"))
        .unwrap();
    store
        .create_session(&synerix::session::Session::new("Second", "m"))
        .unwrap();
    store
        .create_session(&synerix::session::Session::new("Third", "m"))
        .unwrap();

    let sessions = store.list_sessions().unwrap();
    assert_eq!(sessions.len(), 3);
    // All sessions should be present
    let titles: Vec<&str> = sessions.iter().map(|s| s.title.as_str()).collect();
    assert!(titles.contains(&"First"));
    assert!(titles.contains(&"Second"));
    assert!(titles.contains(&"Third"));
}

#[test]
fn test_session_file_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("pipeline_test.db");

    // Create and populate
    {
        let store = synerix::session::SessionStore::open(&db_path).unwrap();
        let session = synerix::session::Session::new("Persistent Session", "model");
        store.create_session(&session).unwrap();
        store
            .save_message(&synerix::session::StoredMessage::user(
                &session.id,
                "persisted msg",
            ))
            .unwrap();
    }

    // Re-open and verify
    {
        let store = synerix::session::SessionStore::open(&db_path).unwrap();
        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].title, "Persistent Session");

        let messages = store.load_messages(&sessions[0].id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "persisted msg");
    }
}

// ── Context Manager Token Budget and Trimming ──────────────

#[test]
fn test_context_manager_basic_flow() {
    use synerix::agent::context::{ContextManager, TokenBudget};

    let budget = TokenBudget::new(128_000);
    let mut ctx = ContextManager::new(budget);

    assert_eq!(ctx.current_tokens(), 0);
    assert!(ctx.messages().is_empty());
    assert_eq!(ctx.usage_ratio(), 0.0);

    ctx.push(synerix::llm::types::ChatMessage::system("You are helpful."));
    ctx.push(synerix::llm::types::ChatMessage::user("Hello"));
    ctx.push(synerix::llm::types::ChatMessage::assistant("Hi there!"));

    assert_eq!(ctx.messages().len(), 3);
    assert!(ctx.current_tokens() > 0);
    assert!(ctx.usage_ratio() > 0.0);
    assert!(ctx.usage_ratio() < 1.0);
}

#[test]
fn test_context_manager_token_budget_properties() {
    use synerix::agent::context::TokenBudget;

    let budget = TokenBudget::new(128_000);
    assert_eq!(budget.total, 128_000);
    assert_eq!(budget.system_prompt, 2000);
    assert_eq!(budget.tools_schema, 3000);
    assert_eq!(budget.reserved, 4096);
    assert_eq!(budget.available, 128_000 - 2000 - 3000 - 4096);
}

#[test]
fn test_context_manager_trims_on_overflow() {
    use synerix::agent::context::{ContextManager, TokenBudget};
    use synerix::llm::types::ChatMessage;

    // Small budget to trigger trimming quickly
    let budget = TokenBudget {
        total: 500,
        system_prompt: 50,
        tools_schema: 50,
        reserved: 100,
        available: 300,
    };

    let mut ctx = ContextManager::new(budget);

    // Push many messages to exceed budget
    for i in 0..30 {
        ctx.push(ChatMessage::user(&format!(
            "Message number {} with enough text to consume tokens in the budget",
            i
        )));
    }

    // Should have been trimmed
    assert!(
        ctx.current_tokens() < 500,
        "Tokens {} should be under budget 500",
        ctx.current_tokens()
    );
    assert!(
        ctx.messages().len() < 30,
        "Messages {} should be less than 30 after trimming",
        ctx.messages().len()
    );
}

#[test]
fn test_context_manager_preserves_system_during_trim() {
    use synerix::agent::context::{ContextManager, TokenBudget};
    use synerix::llm::types::{ChatMessage, MessageRole};

    let budget = TokenBudget {
        total: 500,
        system_prompt: 50,
        tools_schema: 50,
        reserved: 100,
        available: 300,
    };

    let mut ctx = ContextManager::new(budget);
    ctx.push(ChatMessage::system("Important system prompt"));

    for i in 0..50 {
        ctx.push(ChatMessage::user(&format!(
            "Filler message {} to trigger trimming in the context manager",
            i
        )));
    }

    // System message must be preserved
    let has_system = ctx
        .messages()
        .iter()
        .any(|m| matches!(m.role, MessageRole::System));
    assert!(has_system, "System message must survive trimming");
}

#[test]
fn test_context_manager_clear() {
    use synerix::agent::context::{ContextManager, TokenBudget};
    use synerix::llm::types::ChatMessage;

    let budget = TokenBudget::new(100_000);
    let mut ctx = ContextManager::new(budget);

    ctx.push(ChatMessage::user("msg1"));
    ctx.push(ChatMessage::assistant("msg2"));
    assert_eq!(ctx.messages().len(), 2);

    ctx.clear();
    assert_eq!(ctx.messages().len(), 0);
    assert_eq!(ctx.current_tokens(), 0);
}

#[test]
fn test_context_manager_count_by_role() {
    use synerix::agent::context::{ContextManager, TokenBudget};
    use synerix::llm::types::ChatMessage;

    let budget = TokenBudget::new(100_000);
    let mut ctx = ContextManager::new(budget);

    ctx.push(ChatMessage::system("sys"));
    ctx.push(ChatMessage::user("u1"));
    ctx.push(ChatMessage::user("u2"));
    ctx.push(ChatMessage::assistant("a1"));
    ctx.push(ChatMessage::tool_result("t1".into(), "result".into()));

    let (sys, usr, asst, tool) = ctx.count_by_role();
    assert_eq!(sys, 1);
    assert_eq!(usr, 2);
    assert_eq!(asst, 1);
    assert_eq!(tool, 1);
}

#[test]
fn test_context_manager_budget_update() {
    use synerix::agent::context::TokenBudget;

    let mut budget = TokenBudget::new(100_000);
    let original_available = budget.available;

    budget.update_from_actuals(5000, 8000);
    assert_eq!(budget.system_prompt, 5000);
    assert_eq!(budget.tools_schema, 8000);
    assert_eq!(budget.available, 100_000 - 5000 - 8000 - 4096);
    assert_ne!(budget.available, original_available);
}

// ── Workflow Definition Parsing and Validation ──────────────

#[test]
fn test_workflow_parse_yaml_and_validate() {
    let yaml = r#"
name: full-pipeline-test
description: Integration test workflow
version: "2.0"
steps:
  - id: generate
    agent_role: coder
    prompt: "Write a function for {{task}}"
    output_variable: code_output
  - id: review
    agent_role: reviewer
    prompt: "Review: {{code_output}}"
    depends_on: [generate]
    condition: code_output
    output_variable: review_result
  - id: test
    agent_role: tester
    prompt: "Test the code from {{generate}}"
    depends_on: [generate]
variables:
  task: "binary search"
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    assert_eq!(wf.name, "full-pipeline-test");
    assert_eq!(wf.description, "Integration test workflow");
    assert_eq!(wf.version, "2.0");
    assert_eq!(wf.steps.len(), 3);
    assert_eq!(wf.variables.get("task").unwrap(), "binary search");

    // Validate DAG
    assert!(wf.validate_dag().is_ok());
}

#[test]
fn test_workflow_parse_toml_and_validate() {
    let toml_str = r#"
name = "toml-workflow"
description = "Parsed from TOML"

[[steps]]
id = "step1"
agent_role = "coder"
prompt = "Do something"

[[steps]]
id = "step2"
agent_role = "reviewer"
prompt = "Review step1"
depends_on = ["step1"]
"#;
    let wf = synerix::workflow::parse_workflow_toml(toml_str).unwrap();
    assert_eq!(wf.name, "toml-workflow");
    assert_eq!(wf.steps.len(), 2);
    assert!(wf.validate_dag().is_ok());
}

#[test]
fn test_workflow_cycle_detection() {
    let yaml = r#"
name: cyclic
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [b]
  - id: b
    agent_role: reviewer
    prompt: "B"
    depends_on: [a]
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    assert!(wf.validate_dag().is_err());
}

#[test]
fn test_workflow_self_dependency() {
    let yaml = r#"
name: self-dep
steps:
  - id: a
    agent_role: coder
    prompt: "A"
    depends_on: [a]
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    assert!(wf.validate_dag().is_err());
}

#[test]
fn test_workflow_step_defaults() {
    let yaml = r#"
name: defaults-test
steps:
  - id: s1
    agent_role: coder
    prompt: "minimal"
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    assert_eq!(wf.steps.len(), 1);
    assert!(wf.steps[0].depends_on.is_empty());
    assert!(wf.steps[0].condition.is_none());
    assert!(wf.steps[0].output_variable.is_none());
    assert!(wf.steps[0].timeout_secs.is_none());
    assert!(wf.steps[0].retry_count.is_none());
    assert!(wf.variables.is_empty());
}

#[test]
fn test_workflow_step_with_retry_and_timeout() {
    let yaml = r#"
name: retry-test
steps:
  - id: flaky
    agent_role: coder
    prompt: "Do something flaky"
    retry_count: 3
    retry_delay_ms: 500
    timeout_secs: 30
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    assert_eq!(wf.steps[0].retry_count, Some(3));
    assert_eq!(wf.steps[0].retry_delay_ms, Some(500));
    assert_eq!(wf.steps[0].timeout_secs, Some(30));
}

#[test]
fn test_workflow_runner_creation_with_dag() {
    let yaml = r#"
name: runner-test
steps:
  - id: code
    agent_role: coder
    prompt: "Write {{language}}"
  - id: review
    agent_role: reviewer
    prompt: "Review: {{code_output}}"
    depends_on: [code]
    output_variable: review_result
variables:
  language: Rust
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    let swarm = synerix::agent::multi::AgentSwarm::new();
    let runner = synerix::workflow::WorkflowRunner::new(wf, swarm).unwrap();

    // Verify runner setup
    assert_eq!(runner.variables.get("language").unwrap(), "Rust");

    let status = runner.status();
    assert_eq!(status.total_steps, 2);
    assert_eq!(status.completed, 0);

    // Initially, only the first step should be executable
    let executable = runner.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].id, "code");
}

#[test]
fn test_workflow_runner_variable_interpolation() {
    let yaml = r#"
name: interp-test
steps:
  - id: s1
    agent_role: coder
    prompt: "Build {{tool}} for {{project}}"
variables:
  tool: "compiler"
  project: "synerix"
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    let swarm = synerix::agent::multi::AgentSwarm::new();
    let runner = synerix::workflow::WorkflowRunner::new(wf, swarm).unwrap();

    let resolved = runner.resolve_prompt("Build {{tool}} for {{project}}");
    assert_eq!(resolved, "Build compiler for synerix");
}

#[test]
fn test_workflow_runner_condition_evaluation() {
    let yaml = r#"
name: cond-test
steps:
  - id: s1
    agent_role: coder
    prompt: "A"
"#;
    let wf = synerix::workflow::parse_workflow(yaml).unwrap();
    let swarm = synerix::agent::multi::AgentSwarm::new();
    let mut runner = synerix::workflow::WorkflowRunner::new(wf, swarm).unwrap();

    // No variable set
    assert!(!runner.evaluate_condition("missing_var"));

    // Set variable
    runner
        .variables
        .insert("status".into(), "ready".into());
    assert!(runner.evaluate_condition("status"));
    assert!(!runner.evaluate_condition("!status"));
    assert!(runner.evaluate_condition("status != 'error'"));
    assert!(runner.evaluate_condition("status contains 'ready'"));
    assert!(runner.evaluate_condition("status starts_with 're'"));
}

#[test]
fn test_workflow_builtin_definitions() {
    let cr = synerix::workflow::code_review_workflow();
    assert_eq!(cr.name, "code-review");
    assert!(!cr.steps.is_empty());

    let ref_wf = synerix::workflow::refactor_workflow();
    assert_eq!(ref_wf.name, "refactor");
    assert!(!ref_wf.steps.is_empty());

    let dbg = synerix::workflow::debug_workflow();
    assert_eq!(dbg.name, "debug");
    assert!(!dbg.steps.is_empty());
}

#[test]
fn test_workflow_invalid_yaml_returns_error() {
    let result = synerix::workflow::parse_workflow("not valid: [[[yaml");
    assert!(result.is_err());
}

// ── End-to-End Pipeline Flow ───────────────────────────────

#[test]
fn test_full_pipeline_config_to_tools_to_context() {
    // Step 1: Load config
    let settings = synerix::config::Settings::load().unwrap();

    // Step 2: Initialize tool registry
    let mut registry = synerix::tools::ToolRegistry::new();
    synerix::tools::builtin::register_builtins(&mut registry);
    assert_eq!(registry.len(), 5);

    // Step 3: Get tool schemas (for LLM context)
    let schemas = registry.all_schemas();
    assert!(!schemas.is_empty());

    // Step 4: Create context manager with budget from config
    use synerix::agent::context::{ContextManager, TokenBudget};
    let budget = TokenBudget::new(settings.llm.context_window);
    let mut ctx = ContextManager::new(budget);

    // Step 5: Build system prompt with tool info
    ctx.push(synerix::llm::types::ChatMessage::system(
        "You are a helpful coding assistant.",
    ));

    // Step 6: Simulate a conversation
    ctx.push(synerix::llm::types::ChatMessage::user(
        "Write a hello world program",
    ));
    ctx.push(synerix::llm::types::ChatMessage::assistant(
        "Here is a hello world program...",
    ));

    assert!(ctx.current_tokens() > 0);
    assert!(ctx.messages().len() == 3);
    assert!(ctx.usage_ratio() < 1.0);
}

#[tokio::test]
async fn test_full_pipeline_skills_and_session() {
    // Step 1: Load skills
    let dir = tempfile::tempdir().unwrap();
    let skill_content = r#"---
name: e2e-test-skill
description: End-to-end test skill
instructions: ""
trigger:
  type: auto_match
  keywords: ["hello", "greet"]
  threshold: 0.5
---
Say hello nicely.
"#;
    std::fs::write(dir.path().join("hello.md"), skill_content).unwrap();

    let skill_registry =
        synerix::skills::SkillRegistry::load_from_dir(dir.path())
            .await
            .unwrap();
    assert_eq!(skill_registry.list_names().len(), 1);

    // Step 2: Create session store
    let store = synerix::session::SessionStore::memory().unwrap();
    let session = synerix::session::Session::new("E2E Test", "test-model");
    store.create_session(&session).unwrap();

    // Step 3: Match skills based on input
    let matched = skill_registry.match_skills("hello world");
    let instructions = skill_registry.build_instructions(&matched);

    // Step 4: Store the conversation
    store
        .save_message(&synerix::session::StoredMessage::user(
            &session.id,
            "hello world",
        ))
        .unwrap();
    store
        .save_message(&synerix::session::StoredMessage::assistant(
            &session.id,
            &format!("Here's the response with skills: {}", instructions),
        ))
        .unwrap();

    // Step 5: Verify the full flow
    let messages = store.load_messages(&session.id).unwrap();
    assert_eq!(messages.len(), 2);
    assert!(messages[1].content.contains("Active Skills"));
}
