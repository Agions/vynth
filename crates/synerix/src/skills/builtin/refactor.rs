//! Refactor built-in skill

use crate::skills::traits::{SkillDef, SkillTrigger};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refactor_skill_name() {
        let skill = refactor_skill();
        assert_eq!(skill.name, "refactor");
    }

    #[test]
    fn refactor_skill_description() {
        let skill = refactor_skill();
        assert!(skill.description.contains("refactoring"));
    }

    #[test]
    fn refactor_skill_trigger_is_auto_match() {
        let skill = refactor_skill();
        match &skill.trigger {
            SkillTrigger::AutoMatch {
                keywords,
                threshold,
            } => {
                assert!(keywords.contains(&"refactor".to_string()));
                assert!(keywords.contains(&"clean up".to_string()));
                assert!(keywords.contains(&"improve".to_string()));
                assert!(keywords.contains(&"restructure".to_string()));
                assert_eq!(keywords.len(), 4);
                assert!((threshold - 0.3).abs() < f32::EPSILON);
            }
            _ => panic!("Expected AutoMatch trigger"),
        }
    }

    #[test]
    fn refactor_skill_instructions_contain_guidelines() {
        let skill = refactor_skill();
        assert!(skill.instructions.contains("Preserve behavior"));
        assert!(skill.instructions.contains("Small steps"));
        assert!(skill.instructions.contains("DRY"));
        assert!(skill.instructions.contains("Naming"));
        assert!(skill.instructions.contains("Dependencies"));
    }

    #[test]
    fn refactor_skill_instructions_contain_process() {
        let skill = refactor_skill();
        assert!(skill.instructions.contains("code smells"));
        assert!(skill.instructions.contains("refactoring plan"));
        assert!(skill.instructions.contains("incrementally"));
    }

    #[test]
    fn refactor_skill_required_tools() {
        let skill = refactor_skill();
        assert_eq!(skill.required_tools.len(), 4);
        assert!(skill.required_tools.contains(&"file_read".to_string()));
        assert!(skill.required_tools.contains(&"file_write".to_string()));
        assert!(skill.required_tools.contains(&"search".to_string()));
        assert!(skill.required_tools.contains(&"patch".to_string()));
    }

    #[test]
    fn refactor_skill_no_required_mcp() {
        let skill = refactor_skill();
        assert!(skill.required_mcp.is_empty());
    }

    #[test]
    fn refactor_skill_source_path_is_none() {
        let skill = refactor_skill();
        assert!(skill.source_path.is_none());
    }

    #[test]
    fn refactor_skill_has_more_tools_than_code_review() {
        let refactor = refactor_skill();
        // Refactor needs file_write and patch in addition to file_read and search
        assert!(refactor.required_tools.contains(&"file_write".to_string()));
        assert!(refactor.required_tools.contains(&"patch".to_string()));
    }
}
