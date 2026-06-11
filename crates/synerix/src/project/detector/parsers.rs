//! Language and framework detection parsers.
//!
//! Provides extension-based and marker-file-based detection of programming
//! languages, project types, and well-known config files.
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::project::types::ProjectType;

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

/// Scan for well-known config files in the project root.
pub(super) fn detect_config_files(root: &Path) -> Vec<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn detect_languages_rust_by_extension() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Rust"));
    }

    #[tokio::test]
    async fn detect_languages_python_by_extension() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("app.py"), "print('hi')").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Python"));
    }

    #[tokio::test]
    async fn detect_languages_js_ts_by_extension() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("index.ts"), "").unwrap();
        fs::write(dir.path().join("app.jsx"), "").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("JavaScript/TypeScript"));
    }

    #[tokio::test]
    async fn detect_languages_go_by_extension() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.go"), "").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Go"));
    }

    #[tokio::test]
    async fn detect_languages_by_marker_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Rust"));
    }

    #[tokio::test]
    async fn detect_languages_empty_dir() {
        let dir = TempDir::new().unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.is_empty());
    }

    #[tokio::test]
    async fn detect_languages_subdir() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("src");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("lib.rs"), "").unwrap();
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Rust"));
    }

    #[test]
    fn detect_project_type_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let langs = HashSet::new();
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Rust);
    }

    #[test]
    fn detect_project_type_node() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let langs = HashSet::new();
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Node);
    }

    #[test]
    fn detect_project_type_python() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        let langs = HashSet::new();
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Python);
    }

    #[test]
    fn detect_project_type_go() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "").unwrap();
        let langs = HashSet::new();
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Go);
    }

    #[test]
    fn detect_project_type_mixed() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::write(dir.path().join("package.json"), "").unwrap();
        let langs = HashSet::new();
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Mixed);
    }

    #[test]
    fn detect_project_type_unknown() {
        let dir = TempDir::new().unwrap();
        let langs = HashSet::new();
        assert_eq!(
            detect_project_type(dir.path(), &langs),
            ProjectType::Unknown
        );
    }

    #[test]
    fn detect_project_type_from_languages_fallback() {
        let dir = TempDir::new().unwrap();
        let mut langs = HashSet::new();
        langs.insert("Python".to_string());
        assert_eq!(detect_project_type(dir.path(), &langs), ProjectType::Python);
    }

    #[test]
    fn detect_config_files_finds_cargo() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let found = detect_config_files(dir.path());
        assert!(found.iter().any(|p| p.ends_with("Cargo.toml")));
    }

    #[test]
    fn detect_config_files_empty() {
        let dir = TempDir::new().unwrap();
        let found = detect_config_files(dir.path());
        assert!(found.is_empty());
    }
}
