//! Code review built-in skill
// TODO: Builtin skills — not yet wired
#![allow(dead_code)]

use crate::skills::traits::{SkillDef, SkillTrigger};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_review_skill_name() {
        let skill = code_review_skill();
        assert_eq!(skill.name, "code-review");
    }

    #[test]
    fn code_review_skill_description() {
        let skill = code_review_skill();
        assert!(skill.description.contains("code review"));
    }

    #[test]
    fn code_review_skill_trigger_is_auto_match() {
        let skill = code_review_skill();
        match &skill.trigger {
            SkillTrigger::AutoMatch {
                keywords,
                threshold,
            } => {
                assert!(keywords.contains(&"review".to_string()));
                assert!(keywords.contains(&"code review".to_string()));
                assert!(keywords.contains(&"check code".to_string()));
                assert!(keywords.contains(&"audit".to_string()));
                assert_eq!(keywords.len(), 4);
                assert!((threshold - 0.3).abs() < f32::EPSILON);
            }
            _ => panic!("Expected AutoMatch trigger"),
        }
    }

    #[test]
    fn code_review_skill_instructions_contain_guidelines() {
        let skill = code_review_skill();
        assert!(skill.instructions.contains("Correctness"));
        assert!(skill.instructions.contains("Security"));
        assert!(skill.instructions.contains("Performance"));
        assert!(skill.instructions.contains("Readability"));
        assert!(skill.instructions.contains("Maintainability"));
    }

    #[test]
    fn code_review_skill_required_tools() {
        let skill = code_review_skill();
        assert_eq!(skill.required_tools.len(), 2);
        assert!(skill.required_tools.contains(&"file_read".to_string()));
        assert!(skill.required_tools.contains(&"search".to_string()));
    }

    #[test]
    fn code_review_skill_no_required_mcp() {
        let skill = code_review_skill();
        assert!(skill.required_mcp.is_empty());
    }

    #[test]
    fn code_review_skill_source_path_is_none() {
        let skill = code_review_skill();
        assert!(skill.source_path.is_none());
    }
}
