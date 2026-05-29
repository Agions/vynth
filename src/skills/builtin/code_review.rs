//! Code review built-in skill

use crate::skills::trait_def::{SkillDef, SkillTrigger};

pub fn code_review_skill() -> SkillDef {
    SkillDef {
        name: "code-review".to_string(),
        description: "Perform a thorough code review".to_string(),
        trigger: SkillTrigger::AutoMatch {
            keywords: vec![
                "review".to_string(),
                "code review".to_string(),
                "check code".to_string(),
                "audit".to_string(),
            ],
            threshold: 0.3,
        },
        instructions: r#"## Code Review Guidelines

When reviewing code, focus on:
1. **Correctness**: Logic errors, edge cases, off-by-one
2. **Security**: Input validation, injection, secrets in code
3. **Performance**: O(n²) loops, unnecessary allocations, N+1 queries
4. **Readability**: Naming, structure, comments, magic numbers
5. **Maintainability**: DRY, separation of concerns, testability

Output format:
- File:line — Issue description (severity: critical/warning/info)
- Suggest concrete fixes with code snippets"#
            .to_string(),
        required_tools: vec!["file_read".to_string(), "search".to_string()],
        required_mcp: Vec::new(),
        source_path: None,
    }
}
