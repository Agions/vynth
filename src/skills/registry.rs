//! Skill registry — load from directory + condition matching

use std::path::Path;

use crate::error::AppError;
use crate::skills::loader::load_skill_file;
use crate::skills::trait_def::SkillDef;

/// Skill registry
pub struct SkillRegistry {
    skills: Vec<SkillDef>,
}

impl SkillRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    /// Load all skills from a directory (recursively scans *.md files)
    pub async fn load_from_dir(path: &Path) -> Result<Self, AppError> {
        let mut skills = Vec::new();

        if !path.exists() {
            tracing::info!("Skills directory not found: {}", path.display());
            return Ok(Self { skills });
        }

        for entry in walk_dir(path).await {
            if entry.extension().map_or(false, |ext| ext == "md") {
                match load_skill_file(&entry).await {
                    Ok(skill) => {
                        tracing::info!("Loaded skill: {}", skill.name);
                        skills.push(skill);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load skill {}: {}", entry.display(), e);
                    }
                }
            }
        }

        tracing::info!("Loaded {} skills from {}", skills.len(), path.display());
        Ok(Self { skills })
    }

    /// Match skills based on user input text
    pub fn match_skills(&self, input: &str) -> Vec<&SkillDef> {
        let input_lower = input.to_lowercase();

        self.skills
            .iter()
            .filter(|skill| match &skill.trigger {
                crate::skills::trait_def::SkillTrigger::Explicit => false,
                crate::skills::trait_def::SkillTrigger::AutoMatch {
                    keywords,
                    threshold,
                } => {
                    let matches = keywords
                        .iter()
                        .filter(|kw| input_lower.contains(&kw.to_lowercase()))
                        .count();
                    let score = matches as f32 / keywords.len() as f32;
                    score >= *threshold
                }
            })
            .collect()
    }

    /// Build combined instructions from matched skills
    pub fn build_instructions(&self, matched: &[&SkillDef]) -> String {
        if matched.is_empty() {
            return String::new();
        }

        let mut result = String::from("\n\n## Active Skills\n\n");
        for skill in matched {
            result.push_str(&format!("### {}\n{}\n\n", skill.name, skill.instructions));
        }
        result
    }

    /// Get skill by name
    pub fn get(&self, name: &str) -> Option<&SkillDef> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// List all skill names
    pub fn list_names(&self) -> Vec<&str> {
        self.skills.iter().map(|s| s.name.as_str()).collect()
    }
}

/// Simple recursive directory walker
async fn walk_dir(path: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let mut read_dir = match tokio::fs::read_dir(path).await {
        Ok(rd) => rd,
        Err(_) => return result,
    };
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            result.extend(Box::pin(walk_dir(&path)).await);
        } else {
            result.push(path);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::trait_def::{SkillDef, SkillTrigger};

    fn test_skill(name: &str, keywords: Vec<&str>, threshold: f32) -> SkillDef {
        SkillDef {
            name: name.into(),
            description: format!("Test skill {}", name),
            trigger: SkillTrigger::AutoMatch {
                keywords: keywords.into_iter().map(String::from).collect(),
                threshold,
            },
            instructions: format!("Instructions for {}", name),
            required_tools: vec![],
            required_mcp: vec![],
            source_path: None,
        }
    }

    fn explicit_skill(name: &str) -> SkillDef {
        SkillDef {
            name: name.into(),
            description: format!("Explicit skill {}", name),
            trigger: SkillTrigger::Explicit,
            instructions: format!("Instructions for {}", name),
            required_tools: vec![],
            required_mcp: vec![],
            source_path: None,
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = SkillRegistry::new();
        assert!(registry.list_names().is_empty());
    }

    #[test]
    fn test_get_by_name() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .push(test_skill("code_review", vec!["review", "code"], 0.5));
        registry
            .skills
            .push(test_skill("refactor", vec!["refactor", "clean"], 0.5));

        assert!(registry.get("code_review").is_some());
        assert!(registry.get("refactor").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_names() {
        let mut registry = SkillRegistry::new();
        registry.skills.push(test_skill("a", vec!["x"], 0.5));
        registry.skills.push(test_skill("b", vec!["y"], 0.5));

        let names = registry.list_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_match_skills_keyword_match() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .push(test_skill("review", vec!["review", "code"], 0.5));

        let matched = registry.match_skills("please review this code");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "review");
    }

    #[test]
    fn test_match_skills_no_match() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .push(test_skill("review", vec!["review", "code"], 0.5));

        let matched = registry.match_skills("hello world");
        assert!(matched.is_empty());
    }

    #[test]
    fn test_match_skills_threshold() {
        let mut registry = SkillRegistry::new();
        // Requires 100% keyword match
        registry
            .skills
            .push(test_skill("strict", vec!["alpha", "beta"], 1.0));

        // Only 50% match
        let matched = registry.match_skills("alpha");
        assert!(matched.is_empty());

        // 100% match
        let matched = registry.match_skills("alpha beta");
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_match_skills_explicit_excluded() {
        let mut registry = SkillRegistry::new();
        registry.skills.push(explicit_skill("manual_only"));

        let matched = registry.match_skills("manual_only");
        assert!(matched.is_empty());
    }

    #[test]
    fn test_build_instructions_empty() {
        let registry = SkillRegistry::new();
        let instructions = registry.build_instructions(&[]);
        assert!(instructions.is_empty());
    }

    #[test]
    fn test_build_instructions_single() {
        let mut registry = SkillRegistry::new();
        registry
            .skills
            .push(test_skill("review", vec!["review"], 0.5));

        let matched = registry.match_skills("review");
        let instructions = registry.build_instructions(&matched);
        assert!(instructions.contains("Active Skills"));
        assert!(instructions.contains("review"));
        assert!(instructions.contains("Instructions for review"));
    }

    #[test]
    fn test_build_instructions_multiple() {
        let mut registry = SkillRegistry::new();
        registry.skills.push(test_skill("a", vec!["alpha"], 0.5));
        registry.skills.push(test_skill("b", vec!["beta"], 0.5));

        let matched = registry.match_skills("alpha beta");
        let instructions = registry.build_instructions(&matched);
        assert!(instructions.contains("a"));
        assert!(instructions.contains("b"));
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_dir() {
        let registry = SkillRegistry::load_from_dir(Path::new("/nonexistent/path"))
            .await
            .unwrap();
        assert!(registry.list_names().is_empty());
    }

    #[tokio::test]
    async fn test_load_from_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::load_from_dir(dir.path()).await.unwrap();
        assert!(registry.list_names().is_empty());
    }
}
