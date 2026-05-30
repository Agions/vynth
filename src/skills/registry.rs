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
