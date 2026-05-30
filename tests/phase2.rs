//! Phase 2 integration tests — Agent Loop, Tools, Sandbox

use std::path::PathBuf;
use syncode::sandbox::command_preview::RiskLevel;
use syncode::sandbox::CommandPreview;
use syncode::tools::builtin;
use syncode::tools::trait_def::{Tool, ToolContext, ToolResult};

fn test_ctx() -> ToolContext {
    ToolContext {
        working_dir: PathBuf::from("/tmp/syncode_test"),
        sandbox_mode: syncode::config::SandboxMode::Auto,
        approval_handler: None,
    }
}

fn setup_test_dir() -> PathBuf {
    let dir = PathBuf::from("/tmp/syncode_test");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ── File Read Tool ────────────────────────────────────────

#[tokio::test]
async fn test_file_read() {
    let dir = setup_test_dir();
    let test_file = dir.join("read_test.txt");
    std::fs::write(&test_file, "line 1\nline 2\nline 3\n").unwrap();

    let tool = builtin::FileReadTool;
    let args = serde_json::json!({"path": "read_test.txt"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("line 1"));
    assert!(result.output.contains("line 2"));

    std::fs::remove_file(&test_file).unwrap();
}

#[tokio::test]
async fn test_file_read_with_offset_limit() {
    let dir = setup_test_dir();
    let test_file = dir.join("read_offset.txt");
    std::fs::write(&test_file, "a\nb\nc\nd\ne\n").unwrap();

    let tool = builtin::FileReadTool;
    let args = serde_json::json!({"path": "read_offset.txt", "offset": 2, "limit": 2});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("2|b"));
    assert!(result.output.contains("3|c"));
    assert!(!result.output.contains("1|a"));

    std::fs::remove_file(&test_file).unwrap();
}

#[tokio::test]
async fn test_file_read_nonexistent() {
    let tool = builtin::FileReadTool;
    let args = serde_json::json!({"path": "nonexistent_file_xyz.txt"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await;
    assert!(result.is_err());
}

// ── File Write Tool ───────────────────────────────────────

#[tokio::test]
async fn test_file_write() {
    let dir = setup_test_dir();
    let test_file = dir.join("write_test.txt");

    let tool = builtin::FileWriteTool;
    let args = serde_json::json!({"path": "write_test.txt", "content": "hello syncode"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("1 lines"));

    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "hello syncode");

    std::fs::remove_file(&test_file).unwrap();
}

#[tokio::test]
async fn test_file_write_creates_dirs() {
    let dir = setup_test_dir();
    let nested = dir.join("nested/deep/file.txt");

    let tool = builtin::FileWriteTool;
    let args = serde_json::json!({"path": "nested/deep/file.txt", "content": "nested!"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);

    let content = std::fs::read_to_string(&nested).unwrap();
    assert_eq!(content, "nested!");

    std::fs::remove_dir_all(dir.join("nested")).unwrap();
}

#[test]
fn test_file_write_requires_approval() {
    let tool = builtin::FileWriteTool;
    assert!(tool.requires_approval(&serde_json::json!({})));
}

// ── Shell Exec Tool ───────────────────────────────────────

#[tokio::test]
async fn test_shell_exec_basic() {
    let tool = builtin::ShellExecTool;
    let args = serde_json::json!({"command": "echo hello"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.trim() == "hello");
}

#[tokio::test]
async fn test_shell_exec_stderr() {
    let tool = builtin::ShellExecTool;
    let args = serde_json::json!({"command": "echo err >&2"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("[stderr]"));
}

#[tokio::test]
async fn test_shell_exec_failure() {
    let tool = builtin::ShellExecTool;
    let args = serde_json::json!({"command": "false"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(result.is_error);
}

#[test]
fn test_shell_exec_dangerous_detection() {
    let tool = builtin::ShellExecTool;

    assert!(tool.requires_approval(&serde_json::json!({"command": "rm -rf /"})));
    assert!(tool.requires_approval(&serde_json::json!({"command": "mkfs.ext4 /dev/sda"})));
    assert!(!tool.requires_approval(&serde_json::json!({"command": "ls -la"})));
    assert!(!tool.requires_approval(&serde_json::json!({"command": "echo hello"})));
}

// ── Search Tool ───────────────────────────────────────────

#[tokio::test]
async fn test_search_filename() {
    let dir = setup_test_dir();
    let test_file = dir.join("search_me_xyz.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    let tool = builtin::SearchTool;
    let args = serde_json::json!({"pattern": "search_me_xyz*", "mode": "filename"});
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("search_me_xyz.rs"));

    std::fs::remove_file(&test_file).unwrap();
}

// ── Patch Tool ────────────────────────────────────────────

#[tokio::test]
async fn test_patch_find_replace() {
    let dir = setup_test_dir();
    let test_file = dir.join("patch_test.txt");
    std::fs::write(&test_file, "hello world\nfoo bar\nhello again").unwrap();

    let tool = builtin::PatchTool;
    let args = serde_json::json!({
        "path": "patch_test.txt",
        "old_text": "hello world",
        "new_text": "hi universe"
    });
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("1 replacement"));

    let content = std::fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("hi universe"));
    assert!(content.contains("hello again")); // second occurrence not replaced

    std::fs::remove_file(&test_file).unwrap();
}

#[tokio::test]
async fn test_patch_replace_all() {
    let dir = setup_test_dir();
    let test_file = dir.join("patch_all.txt");
    std::fs::write(&test_file, "aaa bbb aaa bbb aaa").unwrap();

    let tool = builtin::PatchTool;
    let args = serde_json::json!({
        "path": "patch_all.txt",
        "old_text": "aaa",
        "new_text": "xxx",
        "replace_all": true
    });
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("3 replacement"));

    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "xxx bbb xxx bbb xxx");

    std::fs::remove_file(&test_file).unwrap();
}

#[tokio::test]
async fn test_patch_not_found() {
    let dir = setup_test_dir();
    let test_file = dir.join("patch_miss.txt");
    std::fs::write(&test_file, "nothing here").unwrap();

    let tool = builtin::PatchTool;
    let args = serde_json::json!({
        "path": "patch_miss.txt",
        "old_text": "nonexistent",
        "new_text": "replacement"
    });
    let ctx = test_ctx();

    let result = tool.execute(args, &ctx).await.unwrap();
    assert!(result.is_error);
    assert!(result.output.contains("not found"));

    std::fs::remove_file(&test_file).unwrap();
}

#[test]
fn test_patch_requires_approval() {
    let tool = builtin::PatchTool;
    assert!(tool.requires_approval(&serde_json::json!({})));
}

// ── Sandbox: Command Preview ──────────────────────────────

#[test]
fn test_command_preview_safe() {
    let preview = CommandPreview::analyze("ls -la");
    assert_eq!(preview.risk_level, RiskLevel::Safe);
    assert!(preview.description.contains("Read-only"));
}

#[test]
fn test_command_preview_critical() {
    let preview = CommandPreview::analyze("rm -rf /");
    assert_eq!(preview.risk_level, RiskLevel::Critical);
    assert!(preview.description.contains("Recursive delete"));
}

#[test]
fn test_command_preview_high_sudo() {
    let preview = CommandPreview::analyze("sudo apt install something");
    assert_eq!(preview.risk_level, RiskLevel::High);
}

#[test]
fn test_command_preview_medium_redirect() {
    let preview = CommandPreview::analyze("echo data > output.txt");
    assert_eq!(preview.risk_level, RiskLevel::Medium);
}

#[test]
fn test_command_preview_low_git() {
    let preview = CommandPreview::analyze("git status");
    assert_eq!(preview.risk_level, RiskLevel::Low);
}

#[test]
fn test_command_preview_display() {
    let preview = CommandPreview::analyze("rm -rf /");
    let display = preview.display();
    assert!(display.contains("🔴"));
    assert!(display.contains("CRITICAL"));
}

// ── Sandbox: Atomic Write ─────────────────────────────────

#[test]
fn test_atomic_write_basic() {
    let dir = std::env::temp_dir().join("syncode_atomic_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("atomic.txt");

    syncode::sandbox::atomic_write(&path, b"atomic content").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "atomic content");

    // Overwrite
    syncode::sandbox::atomic_write(&path, b"new content").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new content");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_atomic_write_with_backup() {
    let dir = std::env::temp_dir().join("syncode_backup_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("backup.txt");

    // First write
    std::fs::write(&path, "original").unwrap();

    // Atomic write with backup
    syncode::sandbox::atomic_write_with_backup(&path, b"modified").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "modified");

    let bak = path.with_extension("syncode.bak");
    assert_eq!(std::fs::read_to_string(&bak).unwrap(), "original");

    std::fs::remove_dir_all(&dir).unwrap();
}

// ── Tool Registry + Schemas ───────────────────────────────

#[test]
fn test_all_tools_have_valid_schemas() {
    let mut registry = syncode::tools::ToolRegistry::new();
    syncode::tools::builtin::register_builtins(&mut registry);

    for name in registry.list_names() {
        let tool = registry.get(name).unwrap();
        let schema = tool.schema();

        // Every tool must have name, description, parameters
        assert!(schema.get("name").is_some(), "Tool {} missing name", name);
        assert!(
            schema.get("description").is_some(),
            "Tool {} missing description",
            name
        );
        assert!(
            schema.get("parameters").is_some(),
            "Tool {} missing parameters",
            name
        );
    }
}

// ── Agent Context Manager ─────────────────────────────────

#[test]
fn test_context_trim_on_overflow() {
    use syncode::agent::context::{ContextManager, TokenBudget};
    use syncode::llm::types::ChatMessage;

    // Small budget to trigger trimming
    let budget = TokenBudget {
        total: 500,
        system_prompt: 50,
        tools_schema: 50,
        reserved: 100,
        available: 300,
    };

    let mut ctx = ContextManager::new(budget);

    // Push many messages to exceed budget
    for i in 0..20 {
        ctx.push(ChatMessage::user(&format!(
            "Message number {} with some extra text to fill tokens",
            i
        )));
    }

    // Context should have been trimmed
    let tokens = ctx.current_tokens();
    assert!(tokens < 500, "Tokens {} should be under budget 500", tokens);
}
