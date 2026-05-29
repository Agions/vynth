//! Refactor built-in skill

use crate::skills::trait_def::{SkillDef, SkillTrigger};

pub fn refactor_skill() -> SkillDef {
    SkillDef {
        name: "refactor".to_string(),
        description: "Code refactoring guidance".to_string(),
        trigger: SkillTrigger::AutoMatch {
            keywords: vec![
                "refactor".to_string(),
                "clean up".to_string(),
                "improve".to_string(),
                "restructure".to_string(),
            ],
            threshold: 0.3,
        },
        instructions: r#"## Refactoring Guidelines

When refactoring code:
1. **Preserve behavior** — All tests must pass before and after
2. **Small steps** — Each change should be atomic and reviewable
3. **DRY** — Extract repeated logic into functions/types
4. **Naming** — Use descriptive names that reveal intent
5. **Dependencies** — Reduce coupling, increase cohesion

Process:
1. Read and understand current code structure
2. Identify code smells (long functions, deep nesting, duplication)
3. Propose refactoring plan with rationale
4. Apply changes incrementally
5. Verify compilation and test pass"#
            .to_string(),
        required_tools: vec![
            "file_read".to_string(),
            "file_write".to_string(),
            "search".to_string(),
            "patch".to_string(),
        ],
        required_mcp: Vec::new(),
        source_path: None,
    }
}
