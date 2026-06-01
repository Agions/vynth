//! Skill file loader — YAML frontmatter + Markdown body

use std::path::Path;

use crate::error::AppError;
use crate::skills::traits::SkillDef;

/// Load a skill from a Markdown file with YAML frontmatter
///
/// Expected format:
/// ```markdown
/// ---
/// name: code-review
/// description: Perform a thorough code review
/// trigger:
///   auto_match:
///     keywords: ["review", "code quality", "refactor"]
///     threshold: 0.5
/// required_tools: ["file_read", "search"]
/// ---
///
/// # Code Review Instructions
///
/// When reviewing code, focus on...
/// ```
pub async fn load_skill_file(path: &Path) -> Result<SkillDef, AppError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::Config(format!("Failed to read skill file: {}", e)))?;

    // Split frontmatter from body
    let (frontmatter, body) = split_frontmatter(&content)?;

    // Parse YAML frontmatter
    let mut skill: SkillDef = serde_yaml_from_str(&frontmatter)
        .map_err(|e| AppError::Config(format!("Invalid skill frontmatter: {}", e)))?;

    // Set the markdown body as instructions (append to any existing instructions)
    if skill.instructions.is_empty() {
        skill.instructions = body.trim().to_string();
    } else {
        skill.instructions = format!("{}\n\n{}", skill.instructions, body.trim());
    }

    skill.source_path = Some(path.to_path_buf());

    Ok(skill)
}

/// Split YAML frontmatter (between --- markers) from markdown body
fn split_frontmatter(content: &str) -> Result<(String, String), AppError> {
    let content = content.trim();

    if !content.starts_with("---") {
        return Err(AppError::Config(
            "Skill file must start with YAML frontmatter (---)".to_string(),
        ));
    }

    let after_first = &content[3..];
    let end_marker = after_first
        .find("\n---")
        .or_else(|| after_first.find("\r\n---"))
        .ok_or_else(|| {
            AppError::Config("Unclosed frontmatter (missing closing ---)".to_string())
        })?;

    let frontmatter = after_first[..end_marker].trim().to_string();
    let body = after_first[end_marker + 4..].to_string();

    Ok((frontmatter, body))
}

/// Simple YAML parser using serde
fn serde_yaml_from_str<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, String> {
    serde_yaml::from_str(s).map_err(|e| format!("Parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::SkillTrigger;

    #[test]
    fn split_frontmatter_valid() {
        let content = "---\nname: test\ndescription: desc\n---\n\n# Body\nSome content";
        let (fm, body) = split_frontmatter(content).unwrap();
        assert_eq!(fm, "name: test\ndescription: desc");
        assert!(body.contains("# Body"));
        assert!(body.contains("Some content"));
    }

    #[test]
    fn split_frontmatter_no_opening_marker() {
        let result = split_frontmatter("name: test\n---\nbody");
        assert!(result.is_err());
    }

    #[test]
    fn split_frontmatter_no_closing_marker() {
        let result = split_frontmatter("---\nname: test\nno closing here");
        assert!(result.is_err());
    }

    #[test]
    fn split_frontmatter_empty_body() {
        let content = "---\nname: test\n---";
        let (fm, body) = split_frontmatter(content).unwrap();
        assert_eq!(fm, "name: test");
        assert!(body.trim().is_empty());
    }

    #[test]
    fn split_frontmatter_with_extra_whitespace() {
        let content = "  ---\nname: test\n---\n\nbody here  ";
        let (fm, body) = split_frontmatter(content).unwrap();
        assert_eq!(fm, "name: test");
        assert!(body.contains("body here"));
    }

    #[test]
    fn serde_yaml_from_str_valid_skill() {
        let yaml = r#"
name: code-review
description: Perform a code review
instructions: ""
trigger:
  type: Explicit
"#;
        let skill: SkillDef = serde_yaml_from_str(yaml).unwrap();
        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.description, "Perform a code review");
        assert!(matches!(skill.trigger, SkillTrigger::Explicit));
        assert!(skill.required_tools.is_empty());
    }

    #[test]
    fn serde_yaml_from_str_invalid() {
        let result: Result<SkillDef, String> = serde_yaml_from_str("{{invalid");
        assert!(result.is_err());
    }

    #[test]
    fn serde_yaml_from_str_auto_match_trigger() {
        let yaml = r#"
name: test
description: test skill
instructions: ""
trigger:
  type: auto_match
  keywords: ["review", "code"]
  threshold: 0.5
"#;
        let skill: SkillDef = serde_yaml_from_str(yaml).unwrap();
        match skill.trigger {
            SkillTrigger::AutoMatch {
                keywords,
                threshold,
            } => {
                assert_eq!(keywords.len(), 2);
                assert_eq!(threshold, 0.5);
            }
            _ => panic!("Expected AutoMatch trigger"),
        }
    }
}
