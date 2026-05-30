//! Skill trait definition

use serde::{Deserialize, Serialize};

/// Skill trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkillTrigger {
    /// User explicit: /skill <name>
    Explicit,
    /// Auto-match based on keywords
    #[serde(rename = "auto_match")]
    AutoMatch {
        keywords: Vec<String>,
        threshold: f32,
    },
}

/// Skill definition (parsed from YAML frontmatter + Markdown body)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    /// Skill name (unique identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Trigger condition
    pub trigger: SkillTrigger,
    /// Instructions injected into system prompt
    pub instructions: String,
    /// Required tools
    #[serde(default)]
    pub required_tools: Vec<String>,
    /// Required MCP servers
    #[serde(default)]
    pub required_mcp: Vec<String>,
    /// File path (for hot-reload)
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_trigger_explicit_serialization_roundtrip() {
        let trigger = SkillTrigger::Explicit;
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("Explicit"));
        let deserialized: SkillTrigger = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, SkillTrigger::Explicit));
    }

    #[test]
    fn skill_trigger_auto_match_serialization_roundtrip() {
        let trigger = SkillTrigger::AutoMatch {
            keywords: vec!["test".into(), "deploy".into()],
            threshold: 0.8,
        };
        let json = serde_json::to_string(&trigger).unwrap();
        let deserialized: SkillTrigger = serde_json::from_str(&json).unwrap();
        match deserialized {
            SkillTrigger::AutoMatch { keywords, threshold } => {
                assert_eq!(keywords, vec!["test", "deploy"]);
                assert!((threshold - 0.8).abs() < f32::EPSILON);
            }
            _ => panic!("Expected AutoMatch"),
        }
    }

    #[test]
    fn skill_trigger_auto_match_with_threshold() {
        let trigger = SkillTrigger::AutoMatch {
            keywords: vec!["rust".into()],
            threshold: 0.5,
        };
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("\"threshold\":0.5"));
    }

    #[test]
    fn skill_trigger_uses_tagged_serde() {
        let trigger = SkillTrigger::Explicit;
        let json = serde_json::to_string(&trigger).unwrap();
        // Uses #[serde(tag = "type")]
        assert!(json.contains("\"type\""));
    }

    #[test]
    fn skill_def_serialization_roundtrip() {
        let def = SkillDef {
            name: "test_skill".to_string(),
            description: "A test skill".to_string(),
            trigger: SkillTrigger::AutoMatch {
                keywords: vec!["build".into()],
                threshold: 0.7,
            },
            instructions: "Run tests".to_string(),
            required_tools: vec!["cargo".into()],
            required_mcp: vec![],
            source_path: Some("/tmp/skill.md".into()),
        };
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: SkillDef = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_skill");
        assert_eq!(deserialized.description, "A test skill");
        assert_eq!(deserialized.instructions, "Run tests");
        assert_eq!(deserialized.required_tools, vec!["cargo"]);
        assert!(deserialized.required_mcp.is_empty());
        // source_path is #[serde(skip)]
        assert!(deserialized.source_path.is_none());
    }

    #[test]
    fn skill_def_default_empty_vecs() {
        let json = r#"{
            "name": "s",
            "description": "d",
            "trigger": {"type": "Explicit"},
            "instructions": "i"
        }"#;
        let def: SkillDef = serde_json::from_str(json).unwrap();
        assert!(def.required_tools.is_empty());
        assert!(def.required_mcp.is_empty());
    }
}
