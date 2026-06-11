//! External skill sources — load skills from multiple locations
//!
//! Supports:
//! - Local directories (recursive scan)
//! - Git repositories (clone + load)
//! - HTTP URLs (download + load)
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::skills::skill_loader::load_skill_file;
use crate::skills::traits::SkillDef;

/// External skill source configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillSource {
    /// Source type: local, git, url
    #[serde(rename = "type")]
    pub source_type: SkillSourceType,
    /// Path or URL
    pub location: String,
    /// Optional branch for git sources
    #[serde(default)]
    pub branch: Option<String>,
    /// Optional glob patterns to include
    #[serde(default)]
    pub include: Vec<String>,
    /// Optional glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSourceType {
    Local,
    Git,
    Url,
}

/// Load skills from multiple external sources
pub async fn load_external_skills(
    sources: &[SkillSource],
    cache_dir: &Path,
) -> Result<Vec<SkillDef>, AppError> {
    let mut all_skills = Vec::new();

    for source in sources {
        match load_from_source(source, cache_dir).await {
            Ok(skills) => {
                tracing::info!(
                    "Loaded {} skills from {} ({})",
                    skills.len(),
                    source.location,
                    source_type_name(&source.source_type)
                );
                all_skills.extend(skills);
            }
            Err(e) => {
                tracing::warn!("Failed to load skills from {}: {}", source.location, e);
            }
        }
    }

    Ok(all_skills)
}

fn source_type_name(t: &SkillSourceType) -> &'static str {
    match t {
        SkillSourceType::Local => "local",
        SkillSourceType::Git => "git",
        SkillSourceType::Url => "url",
    }
}

async fn load_from_source(
    source: &SkillSource,
    cache_dir: &Path,
) -> Result<Vec<SkillDef>, AppError> {
    match source.source_type {
        SkillSourceType::Local => {
            load_from_local(
                Path::new(&source.location),
                &source.include,
                &source.exclude,
            )
            .await
        }
        SkillSourceType::Git => {
            let target_dir = cache_dir.join("git").join(dir_name(&source.location));
            clone_or_update_git(&source.location, source.branch.as_deref(), &target_dir).await?;
            load_from_local(&target_dir, &source.include, &source.exclude).await
        }
        SkillSourceType::Url => {
            let target_file = cache_dir.join("url").join(url_filename(&source.location));
            download_url(&source.location, &target_file).await?;
            let skill = load_skill_file(&target_file).await?;
            Ok(vec![skill])
        }
    }
}

/// Load skills from a local directory (recursive)
async fn load_from_local(
    dir: &Path,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<SkillDef>, AppError> {
    let mut skills = Vec::new();

    for entry in crate::util::walk_dir(dir).await {
        let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext != "md" {
            continue;
        }

        let rel_path = entry.strip_prefix(dir).unwrap_or(&entry).to_string_lossy();

        // Check include patterns
        if !include.is_empty() && !include.iter().any(|p| glob_match(p, &rel_path)) {
            continue;
        }

        // Check exclude patterns
        if exclude.iter().any(|p| glob_match(p, &rel_path)) {
            continue;
        }

        match load_skill_file(&entry).await {
            Ok(skill) => skills.push(skill),
            Err(e) => {
                tracing::warn!("Failed to load skill {}: {}", entry.display(), e);
            }
        }
    }

    Ok(skills)
}

/// Clone or update a git repository
async fn clone_or_update_git(
    url: &str,
    branch: Option<&str>,
    target: &Path,
) -> Result<(), AppError> {
    if target.exists() {
        // Update existing clone
        let output = std::process::Command::new("git")
            .args(["-C", &target.to_string_lossy(), "pull", "--ff-only"])
            .output()
            .map_err(|e| AppError::Config(format!("git pull failed: {}", e)))?;

        if !output.status.success() {
            tracing::warn!("git pull failed for {}, re-cloning", target.display());
            tokio::fs::remove_dir_all(target).await.ok();
        } else {
            return Ok(());
        }
    }

    // Create parent directories
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Config(format!("Failed to create cache dir: {}", e)))?;
    }

    // Clone
    let target_str = target.to_string_lossy().into_owned();
    let mut args = vec!["clone", "--depth", "1"];
    if let Some(b) = branch {
        args.extend(["--branch", b]);
    }
    args.extend([url, &target_str]);

    let output = std::process::Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| AppError::Config(format!("git clone failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Config(format!(
            "git clone failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Download a file from URL
async fn download_url(url: &str, target: &Path) -> Result<(), AppError> {
    if target.exists() {
        tracing::info!("Using cached download: {}", target.display());
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Config(format!("Failed to create cache dir: {}", e)))?;
    }

    // Use curl for downloads (available on all platforms)
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-o", &target.to_string_lossy(), url])
        .output()
        .map_err(|e| AppError::Config(format!("curl failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Config(format!(
            "Download failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Simple glob matching (supports * and **)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Handle ** patterns
    if let Some(rest) = pattern.strip_prefix("**/") {
        // **/foo matches any path ending with foo
        if rest.contains('/') {
            return text.ends_with(rest) || text.contains(rest);
        }
        // **/foo.md matches foo.md at any depth
        return text.ends_with(rest) || text.split('/').any(|part| simple_glob_match(rest, part));
    }

    if let Some(prefix) = pattern.strip_suffix("/**") {
        // foo/** matches anything under foo/
        return text.starts_with(prefix.trim_end_matches('/'));
    }

    if pattern.contains("/**/") {
        let parts: Vec<&str> = pattern.splitn(2, "/**/").collect();
        if parts.len() == 2 {
            return text.starts_with(parts[0]) && simple_glob_match(parts[1], text);
        }
    }

    // Handle simple * patterns
    simple_glob_match(pattern, text)
}

/// Simple single-level glob matching (* only)
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return text.starts_with(prefix) && text.ends_with(suffix);
    }

    text == pattern
}

/// Extract a directory name from a git URL
fn dir_name(url: &str) -> String {
    url.split('/')
        .next_back()
        .unwrap_or("skills")
        .trim_end_matches(".git")
        .to_string()
}

/// Extract filename from URL
fn url_filename(url: &str) -> String {
    url.split('/').next_back().unwrap_or("skill.md").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*.md", "test.md"));
        assert!(glob_match("skills/*.md", "skills/test.md"));
        assert!(glob_match("**/*.md", "deep/nested/test.md"));
        assert!(!glob_match("*.md", "test.yaml"));
    }

    #[test]
    fn test_dir_name() {
        assert_eq!(dir_name("https://github.com/user/skills.git"), "skills");
        assert_eq!(
            dir_name("https://gitee.com/Agions/synerix-skills"),
            "synerix-skills"
        );
    }

    #[test]
    fn test_url_filename() {
        assert_eq!(
            url_filename("https://example.com/skills/code-review.md"),
            "code-review.md"
        );
    }

    #[test]
    fn test_skill_source_deserialize() {
        let yaml = r#"
type: local
location: ~/.config/synerix/skills
"#;
        let source: SkillSource = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(source.source_type, SkillSourceType::Local));
        assert_eq!(source.location, "~/.config/synerix/skills");
    }

    #[test]
    fn test_skill_source_git() {
        let yaml = r#"
type: git
location: https://gitee.com/Agions/synerix-skills.git
branch: main
include:
  - "**/*.md"
"#;
        let source: SkillSource = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(source.source_type, SkillSourceType::Git));
        assert_eq!(source.branch, Some("main".to_string()));
    }

    #[test]
    fn test_skill_source_url() {
        let yaml = r#"
type: url
location: https://example.com/skill.md
"#;
        let source: SkillSource = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(source.source_type, SkillSourceType::Url));
    }
}
