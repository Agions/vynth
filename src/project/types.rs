//! Project types — `ProjectType`, `ProjectInfo`, and `ProjectContext`.

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
