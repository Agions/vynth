//! Integration tests for Syncode core components

use std::sync::Arc;

// ── Settings ──────────────────────────────────────────────

#[test]
fn test_settings_defaults() {
    let settings = syncode::config::Settings::load().unwrap();
    assert_eq!(settings.llm.model, "deepseek-chat");
    assert_eq!(settings.llm.context_window, 128_000);
    assert_eq!(settings.ui.theme, "dark");
    assert_eq!(settings.ui.keymap, "default");
}

#[test]
fn test_settings_toml_roundtrip() {
    let settings = syncode::config::Settings::load().unwrap();
    let toml_str = toml::to_string(&settings).unwrap();
    let parsed: syncode::config::Settings = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.llm.model, settings.llm.model);
    assert_eq!(parsed.ui.theme, settings.ui.theme);
}

// ── App State Machine ─────────────────────────────────────

#[test]
fn test_app_creation() {
    let settings = syncode::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let app = syncode::app::App::new_with_channel(settings, tx, _rx);

    assert_eq!(app.mode, syncode::app::InputMode::Normal);
    assert!(!app.should_quit);
    assert!(app.chat_state.messages.is_empty());
    assert!(!app.chat_state.is_streaming);
}

#[test]
fn test_input_mode_transitions() {
    let settings = syncode::config::Settings::load().unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = syncode::app::App::new_with_channel(settings, tx, _rx);

    assert_eq!(app.mode, syncode::app::InputMode::Normal);

    // Simulate mode transitions
    app.mode = syncode::app::InputMode::Insert;
    assert_eq!(app.mode, syncode::app::InputMode::Insert);

    app.mode = syncode::app::InputMode::Command;
    assert_eq!(app.mode, syncode::app::InputMode::Command);

    app.mode = syncode::app::InputMode::Normal;
    assert_eq!(app.mode, syncode::app::InputMode::Normal);
}

// ── Tool Registry ─────────────────────────────────────────

#[test]
fn test_tool_registry() {
    let mut registry = syncode::tools::ToolRegistry::new();
    syncode::tools::builtin::register_builtins(&mut registry);

    let names = registry.list_names();
    assert!(names.contains(&"file_read"));
    assert!(names.contains(&"file_write"));
    assert!(names.contains(&"shell_exec"));
    assert!(names.contains(&"search"));
    assert!(names.contains(&"patch"));
    assert_eq!(names.len(), 5);
}

#[test]
fn test_tool_schemas() {
    let mut registry = syncode::tools::ToolRegistry::new();
    syncode::tools::builtin::register_builtins(&mut registry);

    let schemas = registry.all_schemas();
    assert_eq!(schemas.len(), 5);

    for schema in &schemas {
        assert!(!schema.function.name.is_empty());
        assert!(!schema.function.description.is_empty());
        assert_eq!(schema.schema_type, "function");
    }
}

// ── Skill Registry ────────────────────────────────────────

#[test]
fn test_builtin_skills() {
    let code_review = syncode::skills::builtin::code_review_skill();
    assert_eq!(code_review.name, "code-review");
    assert!(!code_review.instructions.is_empty());

    let refactor = syncode::skills::builtin::refactor_skill();
    assert_eq!(refactor.name, "refactor");
    assert!(!refactor.instructions.is_empty());
}

// ── Session Store (SQLite) ────────────────────────────────

#[test]
fn test_session_store_memory() {
    let store = syncode::session::SessionStore::memory().unwrap();

    let session = syncode::session::Session::new("Test Session", "deepseek-chat");
    store.create_session(&session).unwrap();

    let sessions = store.list_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, "Test Session");
}

#[test]
fn test_message_storage() {
    let store = syncode::session::SessionStore::memory().unwrap();

    let session = syncode::session::Session::new("Chat Test", "deepseek-chat");
    store.create_session(&session).unwrap();

    let msg = syncode::session::StoredMessage::user(&session.id, "Hello");
    store.save_message(&msg).unwrap();

    let msg2 = syncode::session::StoredMessage::assistant(&session.id, "Hi there!");
    store.save_message(&msg2).unwrap();

    let messages = store.load_messages(&session.id).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "Hello");
    assert_eq!(messages[1].content, "Hi there!");
}

// ── Context Manager ───────────────────────────────────────

#[test]
fn test_context_manager() {
    use syncode::agent::context::{ContextManager, TokenBudget};

    let budget = TokenBudget::new(128_000);
    let mut ctx = ContextManager::new(budget);

    assert_eq!(ctx.current_tokens(), 0);
    assert!(ctx.messages().is_empty());

    ctx.push(syncode::llm::types::ChatMessage::user("Hello"));
    assert!(!ctx.messages().is_empty());
    assert!(ctx.current_tokens() > 0);
}

#[test]
fn test_token_budget() {
    use syncode::agent::context::TokenBudget;

    let budget = TokenBudget::new(128_000);
    assert_eq!(budget.total, 128_000);
    assert!(budget.available > 0);
    assert!(budget.available < budget.total);
}

// ── LLM Types ─────────────────────────────────────────────

#[test]
fn test_chat_message_types() {
    let sys = syncode::llm::types::ChatMessage::system("You are helpful.");
    assert_eq!(sys.role, syncode::llm::types::MessageRole::System);

    let user = syncode::llm::types::ChatMessage::user("Hello");
    assert_eq!(user.role, syncode::llm::types::MessageRole::User);

    let asst = syncode::llm::types::ChatMessage::assistant("Hi!");
    assert_eq!(asst.role, syncode::llm::types::MessageRole::Assistant);
}

#[test]
fn test_chat_message_to_json() {
    let msg = syncode::llm::types::ChatMessage::user("test message");
    let json = msg.to_json();
    assert_eq!(json["role"], "user");
    assert_eq!(json["content"], "test message");
}

// ── Error Types ───────────────────────────────────────────

#[test]
fn test_error_display() {
    let err = syncode::error::AppError::Config("bad config".to_string());
    assert!(err.to_string().contains("bad config"));

    let err = syncode::error::AppError::ToolNotFound("my_tool".to_string());
    assert!(err.to_string().contains("my_tool"));
}

// ── Sandbox ───────────────────────────────────────────────

#[test]
fn test_command_preview() {
    let preview = syncode::sandbox::CommandPreview::analyze("ls -la");
    assert_eq!(preview.command, "ls -la");

    let dangerous = syncode::sandbox::CommandPreview::analyze("rm -rf /");
    assert_eq!(dangerous.command, "rm -rf /");
}

// ── MCP Types ─────────────────────────────────────────────

#[test]
fn test_json_rpc_request() {
    let req = syncode::mcp::types::JsonRpcRequest::new(1, "initialize", None);
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "initialize");
    assert_eq!(req.jsonrpc, "2.0");
}
