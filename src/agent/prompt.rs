//! System prompt builder

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

/// Default system prompt for Syncode
pub fn default_system_prompt() -> String {
    r#"You are Syncode, an AI pair programming assistant running in a terminal.

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
- For multi-step tasks, state your plan first then execute step by step"#.to_string()
}
