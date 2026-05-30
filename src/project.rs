//! Project context detection — auto-detect language, framework, and project type
//!
//! Provides a lightweight scan of the working directory to build a
//! `ProjectInfo` and `ProjectContext` that inform the agent about the codebase
//! it is operating on.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
        let suggested_skills = suggest_skills(&info.project_type);
        let suggested_tools = suggest_tools(&info.project_type);
        let system_prompt_hint = build_prompt_hint(&info);

        Self {
            info,
            suggested_skills,
            suggested_tools,
            system_prompt_hint,
        }
    }
}

// ---------------------------------------------------------------------------
// Public detection functions
// ---------------------------------------------------------------------------

/// Detect project information starting from `dir`.
///
/// If `dir` is `None`, the current working directory is used.
pub fn detect_project(dir: Option<&Path>) -> ProjectInfo {
    let root = dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let root = find_project_root(&root).unwrap_or(root);

    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());

    let languages = detect_languages(&root);
    let project_type = detect_project_type(&root, &languages);
    let has_git = root.join(".git").exists();
    let has_docker = root.join("Dockerfile").exists()
        || root.join("docker-compose.yml").exists()
        || root.join("docker-compose.yaml").exists();
    let has_ci = root.join(".github").exists()
        || root.join(".gitlab-ci.yml").exists()
        || root.join("Jenkinsfile").exists()
        || root.join(".circleci").exists();

    let config_files = detect_config_files(&root);

    ProjectInfo {
        root_dir: root,
        project_type,
        name,
        languages,
        has_git,
        has_docker,
        has_ci,
        config_files,
    }
}

/// Detect programming languages present in the project directory (shallow scan).
pub fn detect_languages(root: &Path) -> HashSet<String> {
    let mut languages = HashSet::new();

    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Check files by extension
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" => { languages.insert("Rust".into()); }
                        "ts" | "tsx" | "js" | "jsx" | "mjs" => { languages.insert("JavaScript/TypeScript".into()); }
                        "py" => { languages.insert("Python".into()); }
                        "go" => { languages.insert("Go".into()); }
                        "java" | "kt" | "kts" => { languages.insert("Java/Kotlin".into()); }
                        "dart" => { languages.insert("Dart".into()); }
                        _ => {}
                    }
                }
            }

            // Check marker files (one level deep)
            if path.is_dir() {
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for s in sub.flatten() {
                        if s.path().is_file() {
                            if let Some(ext) = s.path().extension().and_then(|e| e.to_str()) {
                                match ext {
                                    "rs" => { languages.insert("Rust".into()); }
                                    "ts" | "tsx" | "js" | "jsx" | "mjs" => { languages.insert("JavaScript/TypeScript".into()); }
                                    "py" => { languages.insert("Python".into()); }
                                    "go" => { languages.insert("Go".into()); }
                                    "java" | "kt" | "kts" => { languages.insert("Java/Kotlin".into()); }
                                    "dart" => { languages.insert("Dart".into()); }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Also check well-known project files for language hints
    if root.join("Cargo.toml").exists() {
        languages.insert("Rust".into());
    }
    if root.join("package.json").exists() || root.join("tsconfig.json").exists() {
        languages.insert("JavaScript/TypeScript".into());
    }
    if root.join("pyproject.toml").exists() || root.join("setup.py").exists() || root.join("requirements.txt").exists() {
        languages.insert("Python".into());
    }
    if root.join("go.mod").exists() {
        languages.insert("Go".into());
    }
    if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
        languages.insert("Java/Kotlin".into());
    }
    if root.join("pubspec.yaml").exists() {
        languages.insert("Dart".into());
    }

    languages
}

/// Determine the dominant project type.
pub fn detect_project_type(root: &Path, languages: &HashSet<String>) -> ProjectType {
    // First check marker files for precise detection
    let has_cargo = root.join("Cargo.toml").exists();
    let has_package = root.join("package.json").exists();
    let has_pyproject = root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
        || root.join("requirements.txt").exists();
    let has_go_mod = root.join("go.mod").exists();
    let has_maven = root.join("pom.xml").exists();
    let has_gradle = root.join("build.gradle").exists();
    let has_pubspec = root.join("pubspec.yaml").exists();

    let markers = [has_cargo, has_package, has_pyproject, has_go_mod, has_maven || has_gradle, has_pubspec];
    let count = markers.iter().filter(|&&b| b).count();

    if count > 1 {
        return ProjectType::Mixed;
    }

    if has_cargo {
        return ProjectType::Rust;
    }
    if has_package {
        return ProjectType::Node;
    }
    if has_pyproject {
        return ProjectType::Python;
    }
    if has_go_mod {
        return ProjectType::Go;
    }
    if has_maven || has_gradle {
        return ProjectType::Java;
    }
    if has_pubspec {
        return ProjectType::Flutter;
    }

    // Fall back to language heuristic
    if languages.contains("Rust") {
        return ProjectType::Rust;
    }
    if languages.contains("JavaScript/TypeScript") {
        return ProjectType::Node;
    }
    if languages.contains("Python") {
        return ProjectType::Python;
    }
    if languages.contains("Go") {
        return ProjectType::Go;
    }
    if languages.contains("Java/Kotlin") {
        return ProjectType::Java;
    }
    if languages.contains("Dart") {
        return ProjectType::Flutter;
    }

    ProjectType::Unknown
}

/// Walk upwards from `start` looking for project root markers.
///
/// Returns the first ancestor (inclusive) that contains a known marker file,
/// or `None` if none is found before reaching the filesystem root.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let markers = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "pubspec.yaml",
        ".git",
    ];

    let mut current = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    };

    loop {
        for marker in &markers {
            if current.join(marker).exists() {
                return Some(current);
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Scan for well-known config files in the project root.
fn detect_config_files(root: &Path) -> Vec<PathBuf> {
    let candidates = [
        "Cargo.toml",
        "package.json",
        "tsconfig.json",
        "pyproject.toml",
        "setup.py",
        "requirements.txt",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "pubspec.yaml",
        "Makefile",
        ".editorconfig",
        ".prettierrc",
        "rustfmt.toml",
        ".eslintrc.json",
        "biome.json",
    ];

    candidates
        .iter()
        .map(|c| root.join(c))
        .filter(|p| p.exists())
        .collect()
}

fn suggest_skills(pt: &ProjectType) -> Vec<String> {
    match pt {
        ProjectType::Rust => vec![
            "code_review".into(),
            "refactor".into(),
            "cargo_test".into(),
        ],
        ProjectType::Node => vec![
            "code_review".into(),
            "refactor".into(),
            "npm_test".into(),
        ],
        ProjectType::Python => vec![
            "code_review".into(),
            "refactor".into(),
            "pytest".into(),
        ],
        ProjectType::Go => vec![
            "code_review".into(),
            "refactor".into(),
            "go_test".into(),
        ],
        ProjectType::Java => vec![
            "code_review".into(),
            "refactor".into(),
        ],
        ProjectType::Flutter => vec![
            "code_review".into(),
            "refactor".into(),
        ],
        ProjectType::Mixed | ProjectType::Unknown => vec![
            "code_review".into(),
            "refactor".into(),
        ],
    }
}

fn suggest_tools(pt: &ProjectType) -> Vec<String> {
    match pt {
        ProjectType::Rust => vec!["cargo".into(), "rustfmt".into(), "clippy".into()],
        ProjectType::Node => vec!["npm".into(), "pnpm".into(), "eslint".into()],
        ProjectType::Python => vec!["pip".into(), "ruff".into(), "pytest".into()],
        ProjectType::Go => vec!["go".into(), "gofmt".into(), "golangci-lint".into()],
        ProjectType::Java => vec!["mvn".into(), "gradle".into()],
        ProjectType::Flutter => vec!["flutter".into(), "dart".into()],
        ProjectType::Mixed | ProjectType::Unknown => vec![],
    }
}

fn build_prompt_hint(info: &ProjectInfo) -> String {
    let langs = info
        .languages
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "Project: {} ({}) | Languages: {} | Git: {} | Docker: {} | CI: {}",
        info.name,
        info.project_type,
        if langs.is_empty() { "unknown".into() } else { langs },
        if info.has_git { "yes" } else { "no" },
        if info.has_docker { "yes" } else { "no" },
        if info.has_ci { "yes" } else { "no" },
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary directory with the given files for testing.
    fn make_temp_project(files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create temp dir");
        for f in files {
            let path = dir.path().join(f);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&path, "").expect("write temp file");
        }
        dir
    }

    #[test]
    fn test_detect_rust_project() {
        let dir = make_temp_project(&["Cargo.toml", "src/main.rs"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Rust);
        assert!(info.languages.contains("Rust"));
        assert_eq!(info.name, dir.path().file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn test_detect_node_project() {
        let dir = make_temp_project(&["package.json", "src/index.ts"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Node);
        assert!(info.languages.contains("JavaScript/TypeScript"));
    }

    #[test]
    fn test_detect_python_project() {
        let dir = make_temp_project(&["pyproject.toml", "main.py"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Python);
        assert!(info.languages.contains("Python"));
    }

    #[test]
    fn test_detect_go_project() {
        let dir = make_temp_project(&["go.mod", "main.go"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Go);
        assert!(info.languages.contains("Go"));
    }

    #[test]
    fn test_detect_java_project() {
        let dir = make_temp_project(&["pom.xml", "src/main.java"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Java);
    }

    #[test]
    fn test_detect_flutter_project() {
        let dir = make_temp_project(&["pubspec.yaml", "lib/main.dart"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Flutter);
        assert!(info.languages.contains("Dart"));
    }

    #[test]
    fn test_detect_mixed_project() {
        let dir = make_temp_project(&["Cargo.toml", "package.json"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Mixed);
    }

    #[test]
    fn test_detect_unknown_project() {
        let dir = make_temp_project(&["README.txt"]);
        let info = detect_project(Some(dir.path()));
        assert_eq!(info.project_type, ProjectType::Unknown);
    }

    #[test]
    fn test_has_git() {
        let dir = make_temp_project(&["Cargo.toml", ".git/HEAD"]);
        let info = detect_project(Some(dir.path()));
        assert!(info.has_git);
    }

    #[test]
    fn test_has_docker() {
        let dir = make_temp_project(&["Cargo.toml", "Dockerfile"]);
        let info = detect_project(Some(dir.path()));
        assert!(info.has_docker);
    }

    #[test]
    fn test_has_ci_github() {
        let dir = make_temp_project(&["Cargo.toml", ".github/workflows/ci.yml"]);
        let info = detect_project(Some(dir.path()));
        assert!(info.has_ci);
    }

    #[test]
    fn test_has_ci_gitlab() {
        let dir = make_temp_project(&["Cargo.toml", ".gitlab-ci.yml"]);
        let info = detect_project(Some(dir.path()));
        assert!(info.has_ci);
    }

    #[test]
    fn test_find_project_root() {
        let dir = make_temp_project(&["Cargo.toml", "src/main.rs"]);
        let nested = dir.path().join("src");
        let root = find_project_root(&nested);
        assert_eq!(root, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn test_project_context_from_info() {
        let dir = make_temp_project(&["Cargo.toml"]);
        let info = detect_project(Some(dir.path()));
        let ctx = ProjectContext::from_info(info);
        assert!(ctx.suggested_skills.contains(&"code_review".to_string()));
        assert!(ctx.suggested_tools.contains(&"cargo".to_string()));
        assert!(ctx.system_prompt_hint.contains("Rust"));
    }

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Rust.to_string(), "Rust");
        assert_eq!(ProjectType::Node.to_string(), "Node");
        assert_eq!(ProjectType::Mixed.to_string(), "Mixed");
        assert_eq!(ProjectType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_detect_languages_with_setup_py() {
        let dir = make_temp_project(&["setup.py"]);
        let langs = detect_languages(dir.path());
        assert!(langs.contains("Python"));
    }

    #[test]
    fn test_config_files_detected() {
        let dir = make_temp_project(&["Cargo.toml", "Makefile"]);
        let info = detect_project(Some(dir.path()));
        assert!(info.config_files.iter().any(|p| p.ends_with("Cargo.toml")));
        assert!(info.config_files.iter().any(|p| p.ends_with("Makefile")));
    }
}
