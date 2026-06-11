//! System prompt builder — context-aware, role-aware, tool-aware
#![allow(dead_code)]

use crate::agent::roles::AgentRole;
use crate::skills::SkillRegistry;

/// Build the system prompt with skills injection
pub fn build_system_prompt(
    base_instructions: &str,
    skills: &SkillRegistry,
    user_input: &str,
) -> String {
    let mut prompt = String::from(base_instructions);

    // Match and inject skills
    let matched = skills.match_skills(user_input);
    if !matched.is_empty() {
        let instructions = skills.build_instructions(&matched);
        prompt.push_str(&instructions);
    }

    prompt
}

/// Build a role-aware system prompt
pub fn build_role_prompt(role: &AgentRole, project_context: Option<&ProjectContext>) -> String {
    // Pre-allocate: system_prompt + estimated context size
    let base = role.system_prompt();
    let estimated_extra = project_context
        .as_ref()
        .map(|ctx| {
            let mut extra = 128; // section headers overhead
            extra += ctx.language.len() + 20; // "- Language: ...\n"
            extra += ctx.framework.as_ref().map_or(0, |f| f.len() + 18);
            extra += ctx.build_tool.as_ref().map_or(0, |b| b.len() + 18);
            if !ctx.test_framework.is_empty() {
                extra += ctx.test_framework.len() + 24;
            }
            extra += ctx.conventions.iter().map(|c| c.len() + 4).sum::<usize>();
            extra
        })
        .unwrap_or(0);
    let mut prompt = String::with_capacity(base.len() + estimated_extra);
    prompt.push_str(&base);

    // Inject project context if available
    if let Some(ctx) = project_context {
        prompt.push_str("\n\n## Project Context\n");
        prompt.push_str("- Language: ");
        prompt.push_str(&ctx.language);
        prompt.push('\n');
        if let Some(ref framework) = ctx.framework {
            prompt.push_str("- Framework: ");
            prompt.push_str(framework);
            prompt.push('\n');
        }
        if let Some(ref build_tool) = ctx.build_tool {
            prompt.push_str("- Build tool: ");
            prompt.push_str(build_tool);
            prompt.push('\n');
        }
        if !ctx.test_framework.is_empty() {
            prompt.push_str("- Test framework: ");
            prompt.push_str(&ctx.test_framework);
            prompt.push('\n');
        }
        if !ctx.conventions.is_empty() {
            prompt.push_str("\n## Project Conventions\n");
            for conv in &ctx.conventions {
                prompt.push_str("- ");
                prompt.push_str(conv);
                prompt.push('\n');
            }
        }
    }

    prompt
}

/// Build a tool-aware prompt that tells the agent what tools are available
pub fn build_tool_aware_prompt(base: &str, available_tools: &[&str]) -> String {
    // Pre-allocate: base + header + estimated tool lines (20 chars avg per tool)
    let extra = if available_tools.is_empty() {
        0
    } else {
        60 + available_tools.iter().map(|t| t.len() + 6).sum::<usize>()
    };
    let mut prompt = String::with_capacity(base.len() + extra);
    prompt.push_str(base);

    if !available_tools.is_empty() {
        prompt.push_str("\n\n## Available Tools\n");
        prompt.push_str("You have access to these tools:\n");
        for tool in available_tools {
            prompt.push_str("- `");
            prompt.push_str(tool);
            prompt.push_str("`\n");
        }
    }

    prompt
}

/// Project context for prompt building
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub language: String,
    pub framework: Option<String>,
    pub build_tool: Option<String>,
    pub test_framework: String,
    pub conventions: Vec<String>,
    pub recent_files: Vec<String>,
}

impl ProjectContext {
    /// Auto-detect project context from working directory
    pub async fn detect() -> Self {
        let mut ctx = Self::default();

        // Detect language and framework from common files
        if std::path::Path::new("Cargo.toml").exists() {
            ctx.language = "Rust".to_string();
            ctx.build_tool = Some("cargo".to_string());
            ctx.test_framework = "cargo test / #[test]".to_string();
            ctx.conventions
                .push("Use `cargo fmt` for formatting".to_string());
            ctx.conventions
                .push("Use `cargo clippy` for linting".to_string());

            // Check for common frameworks
            if let Ok(cargo_content) = tokio::fs::read_to_string("Cargo.toml").await {
                if cargo_content.contains("actix-web") {
                    ctx.framework = Some("actix-web".to_string());
                } else if cargo_content.contains("axum") {
                    ctx.framework = Some("axum".to_string());
                } else if cargo_content.contains("tokio") {
                    ctx.framework = Some("tokio".to_string());
                }
            }
        } else if std::path::Path::new("package.json").exists() {
            ctx.language = "TypeScript/JavaScript".to_string();
            ctx.build_tool = Some("npm/yarn".to_string());
            ctx.test_framework = "jest / vitest".to_string();
            ctx.conventions
                .push("Use `npm run lint` for linting".to_string());

            if let Ok(pkg_content) = tokio::fs::read_to_string("package.json").await {
                if pkg_content.contains("\"react\"") {
                    ctx.framework = Some("React".to_string());
                } else if pkg_content.contains("\"vue\"") {
                    ctx.framework = Some("Vue".to_string());
                } else if pkg_content.contains("\"next\"") {
                    ctx.framework = Some("Next.js".to_string());
                }
            }
        } else if std::path::Path::new("pyproject.toml").exists()
            || std::path::Path::new("requirements.txt").exists()
            || std::path::Path::new("setup.py").exists()
        {
            ctx.language = "Python".to_string();
            ctx.test_framework = "pytest".to_string();
            ctx.conventions
                .push("Use `black` for formatting".to_string());
            ctx.conventions
                .push("Use `ruff` or `flake8` for linting".to_string());

            if std::path::Path::new("pyproject.toml").exists() {
                ctx.build_tool = Some("pip/poetry".to_string());
            }
        } else if std::path::Path::new("go.mod").exists() {
            ctx.language = "Go".to_string();
            ctx.build_tool = Some("go".to_string());
            ctx.test_framework = "go test".to_string();
            ctx.conventions
                .push("Use `gofmt` for formatting".to_string());
            ctx.conventions
                .push("Use `golangci-lint` for linting".to_string());
        } else if std::path::Path::new("pubspec.yaml").exists() {
            ctx.language = "Dart".to_string();
            ctx.build_tool = Some("flutter".to_string());
            ctx.test_framework = "flutter test".to_string();
            ctx.conventions
                .push("Use `dart format` for formatting".to_string());
        }

        ctx
    }
}

/// Default system prompt for Synerix
pub fn default_system_prompt() -> String {
    r#"You are Synerix, an AI coding assistant running in a terminal.

## Capabilities
- Read, write, and search files in the user's project
- Execute shell commands (with user approval for dangerous operations)
- Apply patches and refactorings
- Use MCP tools from configured servers

## Guidelines
1. **Be concise** — Terminal space is limited
2. **Show, don't tell** — Use tools to make changes rather than describing them
3. **Ask before destructive** — Always confirm before deleting or overwriting files
4. **Explain reasoning** — Briefly explain why you're making a change
5. **Use diffs** — Show clear before/after for code changes

## Response Format
- Use tools to accomplish tasks, not just describe them
- When showing code, use brief inline comments
- For multi-step tasks, state your plan first then execute step by step"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_basic() {
        let skills = SkillRegistry::new();
        let prompt = build_system_prompt("You are a helpful assistant.", &skills, "hello");
        assert_eq!(prompt, "You are a helpful assistant.");
    }

    #[test]
    fn test_build_role_prompt_no_context() {
        let prompt = build_role_prompt(&AgentRole::Coder, None);
        assert!(prompt.contains("software engineer"));
    }

    #[test]
    fn test_build_role_prompt_with_context() {
        let ctx = ProjectContext {
            language: "Rust".to_string(),
            framework: Some("tokio".to_string()),
            build_tool: Some("cargo".to_string()),
            test_framework: "cargo test".to_string(),
            conventions: vec!["Use rustfmt".to_string()],
            recent_files: vec![],
        };
        let prompt = build_role_prompt(&AgentRole::Coder, Some(&ctx));
        assert!(prompt.contains("Rust"));
        assert!(prompt.contains("tokio"));
        assert!(prompt.contains("cargo test"));
        assert!(prompt.contains("rustfmt"));
    }

    #[test]
    fn test_build_tool_aware_prompt() {
        let tools = vec!["file_read", "file_write", "shell_exec"];
        let prompt = build_tool_aware_prompt("Base prompt", &tools);
        assert!(prompt.contains("file_read"));
        assert!(prompt.contains("file_write"));
        assert!(prompt.contains("shell_exec"));
    }

    #[test]
    fn test_project_context_default() {
        let ctx = ProjectContext::default();
        assert!(ctx.language.is_empty());
        assert!(ctx.framework.is_none());
    }
}
