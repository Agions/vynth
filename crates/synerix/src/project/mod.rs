//! Project context detection — auto-detect language, framework, and project type
//!
//! Provides a lightweight scan of the working directory to build a
//! `ProjectInfo` and `ProjectContext` that inform the agent about the codebase
//! it is operating on.
#![allow(unused_imports)]

pub mod detector;
pub mod types;

// Re-export public API for backward compatibility.
pub use detector::{
    detect_languages, detect_project, detect_project_type, find_project_root,
    find_synerix_agents_dir, find_synerix_dir, find_synerix_skills_dir,
};
pub use types::{ProjectContext, ProjectInfo, ProjectType};

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

    #[tokio::test]
    async fn test_detect_rust_project() {
        let dir = make_temp_project(&["Cargo.toml", "src/main.rs"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Rust);
        assert!(info.languages.contains("Rust"));
        assert_eq!(info.name, dir.path().file_name().unwrap().to_str().unwrap());
    }

    #[tokio::test]
    async fn test_detect_node_project() {
        let dir = make_temp_project(&["package.json", "src/index.ts"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Node);
        assert!(info.languages.contains("JavaScript/TypeScript"));
    }

    #[tokio::test]
    async fn test_detect_python_project() {
        let dir = make_temp_project(&["pyproject.toml", "main.py"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Python);
        assert!(info.languages.contains("Python"));
    }

    #[tokio::test]
    async fn test_detect_go_project() {
        let dir = make_temp_project(&["go.mod", "main.go"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Go);
        assert!(info.languages.contains("Go"));
    }

    #[tokio::test]
    async fn test_detect_java_project() {
        let dir = make_temp_project(&["pom.xml", "src/main.java"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Java);
    }

    #[tokio::test]
    async fn test_detect_flutter_project() {
        let dir = make_temp_project(&["pubspec.yaml", "lib/main.dart"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Flutter);
        assert!(info.languages.contains("Dart"));
    }

    #[tokio::test]
    async fn test_detect_mixed_project() {
        let dir = make_temp_project(&["Cargo.toml", "package.json"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Mixed);
    }

    #[tokio::test]
    async fn test_detect_unknown_project() {
        let dir = make_temp_project(&["README.txt"]);
        let info = detect_project(Some(dir.path())).await;
        assert_eq!(info.project_type, ProjectType::Unknown);
    }

    #[tokio::test]
    async fn test_has_git() {
        let dir = make_temp_project(&["Cargo.toml", ".git/HEAD"]);
        let info = detect_project(Some(dir.path())).await;
        assert!(info.has_git);
    }

    #[tokio::test]
    async fn test_has_docker() {
        let dir = make_temp_project(&["Cargo.toml", "Dockerfile"]);
        let info = detect_project(Some(dir.path())).await;
        assert!(info.has_docker);
    }

    #[tokio::test]
    async fn test_has_ci_github() {
        let dir = make_temp_project(&["Cargo.toml", ".github/workflows/ci.yml"]);
        let info = detect_project(Some(dir.path())).await;
        assert!(info.has_ci);
    }

    #[tokio::test]
    async fn test_has_ci_gitlab() {
        let dir = make_temp_project(&["Cargo.toml", ".gitlab-ci.yml"]);
        let info = detect_project(Some(dir.path())).await;
        assert!(info.has_ci);
    }

    #[test]
    fn test_find_project_root() {
        let dir = make_temp_project(&["Cargo.toml", "src/main.rs"]);
        let nested = dir.path().join("src");
        let root = find_project_root(&nested);
        assert_eq!(root, Some(dir.path().to_path_buf()));
    }

    #[tokio::test]
    async fn test_project_context_from_info() {
        let dir = make_temp_project(&["Cargo.toml"]);
        let info = detect_project(Some(dir.path())).await;
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

    #[tokio::test]
    async fn test_detect_languages_with_setup_py() {
        let dir = make_temp_project(&["setup.py"]);
        let langs = detect_languages(dir.path()).await;
        assert!(langs.contains("Python"));
    }

    #[tokio::test]
    async fn test_config_files_detected() {
        let dir = make_temp_project(&["Cargo.toml", "Makefile"]);
        let info = detect_project(Some(dir.path())).await;
        assert!(info.config_files.iter().any(|p| p.ends_with("Cargo.toml")));
        assert!(info.config_files.iter().any(|p| p.ends_with("Makefile")));
    }
}
