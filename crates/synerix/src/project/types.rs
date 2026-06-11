//! Project types — `ProjectType`, `ProjectInfo`, and `ProjectContext`.
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// ProjectType
// ---------------------------------------------------------------------------

/// The dominant project type detected in the working directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Flutter,
    /// Multiple project markers found — likely a monorepo.
    Mixed,
    /// No recognisable project markers found.
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Node => write!(f, "Node"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Java => write!(f, "Java"),
            ProjectType::Flutter => write!(f, "Flutter"),
            ProjectType::Mixed => write!(f, "Mixed"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// ProjectInfo
// ---------------------------------------------------------------------------

/// Static metadata about a detected project.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root_dir: PathBuf,
    pub project_type: ProjectType,
    pub name: String,
    pub languages: HashSet<String>,
    pub has_git: bool,
    pub has_docker: bool,
    pub has_ci: bool,
    pub config_files: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// ProjectContext
// ---------------------------------------------------------------------------

/// Rich project context fed to the agent so it can tailor its behaviour.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub info: ProjectInfo,
    /// Skills that are likely relevant for this project type.
    pub suggested_skills: Vec<String>,
    /// Tools that are likely useful for this project type.
    pub suggested_tools: Vec<String>,
    /// A hint injected into the system prompt about the project.
    pub system_prompt_hint: String,
}

impl ProjectContext {
    /// Build a context from info, populating suggestions automatically.
    pub fn from_info(info: ProjectInfo) -> Self {
        let suggested_skills = super::detector::suggest_skills(&info.project_type);
        let suggested_tools = super::detector::suggest_tools(&info.project_type);
        let system_prompt_hint = super::detector::build_prompt_hint(&info);

        Self {
            info,
            suggested_skills,
            suggested_tools,
            system_prompt_hint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_display() {
        assert_eq!(format!("{}", ProjectType::Rust), "Rust");
        assert_eq!(format!("{}", ProjectType::Node), "Node");
        assert_eq!(format!("{}", ProjectType::Python), "Python");
        assert_eq!(format!("{}", ProjectType::Go), "Go");
        assert_eq!(format!("{}", ProjectType::Java), "Java");
        assert_eq!(format!("{}", ProjectType::Flutter), "Flutter");
        assert_eq!(format!("{}", ProjectType::Mixed), "Mixed");
        assert_eq!(format!("{}", ProjectType::Unknown), "Unknown");
    }

    #[test]
    fn test_project_type_equality() {
        assert_eq!(ProjectType::Rust, ProjectType::Rust);
        assert_ne!(ProjectType::Rust, ProjectType::Python);
    }

    #[test]
    fn test_project_type_clone() {
        let pt = ProjectType::Go;
        let cloned = pt.clone();
        assert_eq!(pt, cloned);
    }

    #[test]
    fn test_project_context_from_info() {
        let info = ProjectInfo {
            root_dir: PathBuf::from("/test"),
            project_type: ProjectType::Rust,
            name: "myapp".into(),
            languages: ["Rust".into()].into_iter().collect(),
            has_git: true,
            has_docker: false,
            has_ci: true,
            config_files: vec![],
        };
        let ctx = ProjectContext::from_info(info);
        assert!(ctx.suggested_skills.contains(&"code_review".into()));
        assert!(ctx.suggested_tools.contains(&"cargo".into()));
        assert!(ctx.system_prompt_hint.contains("myapp"));
        assert!(ctx.system_prompt_hint.contains("Rust"));
    }
}
