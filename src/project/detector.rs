//! Project detection logic — language detection, type inference, root finding.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::types::{ProjectInfo, ProjectType};

// ---------------------------------------------------------------------------
// Public detection functions
// ---------------------------------------------------------------------------

/// Detect project information starting from `dir`.
///
/// If `dir` is `None`, the current working directory is used.
pub async fn detect_project(dir: Option<&Path>) -> ProjectInfo {
    let root = dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let root = find_project_root(&root).unwrap_or(root);

    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());

    let languages = detect_languages(&root).await;
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
pub async fn detect_languages(root: &Path) -> HashSet<String> {
    let mut languages = HashSet::new();

    let mut read_dir = match tokio::fs::read_dir(root).await {
        Ok(rd) => rd,
        Err(_) => return languages,
    };
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();

        // Check files by extension
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => {
                        languages.insert("Rust".into());
                    }
                    "ts" | "tsx" | "js" | "jsx" | "mjs" => {
                        languages.insert("JavaScript/TypeScript".into());
                    }
                    "py" => {
                        languages.insert("Python".into());
                    }
                    "go" => {
                        languages.insert("Go".into());
                    }
                    "java" | "kt" | "kts" => {
                        languages.insert("Java/Kotlin".into());
                    }
                    "dart" => {
                        languages.insert("Dart".into());
                    }
                    _ => {}
                }
            }
        }

        // Check marker files (one level deep)
        if path.is_dir() {
            let mut sub_dir = match tokio::fs::read_dir(&path).await {
                Ok(sd) => sd,
                Err(_) => continue,
            };
            while let Ok(Some(s)) = sub_dir.next_entry().await {
                if s.path().is_file() {
                    if let Some(ext) = s.path().extension().and_then(|e| e.to_str()) {
                        match ext {
                            "rs" => {
                                languages.insert("Rust".into());
                            }
                            "ts" | "tsx" | "js" | "jsx" | "mjs" => {
                                languages.insert("JavaScript/TypeScript".into());
                            }
                            "py" => {
                                languages.insert("Python".into());
                            }
                            "go" => {
                                languages.insert("Go".into());
                            }
                            "java" | "kt" | "kts" => {
                                languages.insert("Java/Kotlin".into());
                            }
                            "dart" => {
                                languages.insert("Dart".into());
                            }
                            _ => {}
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
    if root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
        || root.join("requirements.txt").exists()
    {
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

    let markers = [
        has_cargo,
        has_package,
        has_pyproject,
        has_go_mod,
        has_maven || has_gradle,
        has_pubspec,
    ];
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
        start
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
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

pub(crate) fn suggest_skills(pt: &ProjectType) -> Vec<String> {
    match pt {
        ProjectType::Rust => vec!["code_review".into(), "refactor".into(), "cargo_test".into()],
        ProjectType::Node => vec!["code_review".into(), "refactor".into(), "npm_test".into()],
        ProjectType::Python => vec!["code_review".into(), "refactor".into(), "pytest".into()],
        ProjectType::Go => vec!["code_review".into(), "refactor".into(), "go_test".into()],
        ProjectType::Java => vec!["code_review".into(), "refactor".into()],
        ProjectType::Flutter => vec!["code_review".into(), "refactor".into()],
        ProjectType::Mixed | ProjectType::Unknown => vec!["code_review".into(), "refactor".into()],
    }
}

pub(crate) fn suggest_tools(pt: &ProjectType) -> Vec<String> {
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

pub(crate) fn build_prompt_hint(info: &ProjectInfo) -> String {
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
        if langs.is_empty() {
            "unknown".into()
        } else {
            langs
        },
        if info.has_git { "yes" } else { "no" },
        if info.has_docker { "yes" } else { "no" },
        if info.has_ci { "yes" } else { "no" },
    )
}
