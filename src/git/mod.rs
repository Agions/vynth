//! Git integration for Syncode
//!
//! Provides types and functions for interacting with git repositories,
//! including status, diff, log, branching, staging, and committing.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};

/// The type of change detected in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Added => write!(f, "A"),
            ChangeType::Modified => write!(f, "M"),
            ChangeType::Deleted => write!(f, "D"),
            ChangeType::Renamed => write!(f, "R"),
        }
    }
}

/// A single file change in the working tree or index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
}

/// The full git status of a repository.
#[derive(Debug, Clone)]
pub struct GitStatus {
    /// Current branch name.
    pub branch: String,
    /// Commits ahead of the upstream tracking branch.
    pub ahead: usize,
    /// Commits behind the upstream tracking branch.
    pub behind: usize,
    /// Files staged for commit (in the index).
    pub staged: Vec<FileChange>,
    /// Modified files not yet staged.
    pub unstaged: Vec<FileChange>,
    /// Untracked files.
    pub untracked: Vec<FileChange>,
}

/// A single git commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
}

/// A git branch entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

// ---------------------------------------------------------------------------
// Helper: run a git command in a given directory
// ---------------------------------------------------------------------------

fn run_git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_ok(dir: &Path, args: &[&str]) -> Result<(), String> {
    run_git(dir, args)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Get the full working-tree status of a git repository at `dir`.
pub fn git_status(dir: &Path) -> Result<GitStatus, String> {
    let branch_out = run_git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch_out.trim().to_string();

    // ahead / behind
    let (ahead, behind) = match run_git(
        dir,
        &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
    ) {
        Ok(ab) => {
            let parts: Vec<&str> = ab.trim().split('\t').collect();
            if parts.len() == 2 {
                (
                    parts[0].parse::<usize>().unwrap_or(0),
                    parts[1].parse::<usize>().unwrap_or(0),
                )
            } else {
                (0, 0)
            }
        }
        Err(_) => (0, 0), // no upstream configured
    };

    // Parse porcelain status
    let status_raw = run_git(dir, &["status", "--porcelain=v1"])?;

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for line in status_raw.lines() {
        if line.len() < 3 {
            continue;
        }
        let x = line.chars().nth(0).unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        let path_str = line[3..].trim();
        let path = PathBuf::from(path_str);

        match (x, y) {
            // Unmerged / conflict — treat as modified in both
            ('U', _) | (_, 'U') | ('D', 'D') | ('A', 'A') => {
                staged.push(FileChange {
                    path: path.clone(),
                    change_type: ChangeType::Modified,
                });
                unstaged.push(FileChange {
                    path,
                    change_type: ChangeType::Modified,
                });
            }
            // New file (untracked)
            ('?', '?') => {
                untracked.push(FileChange {
                    path,
                    change_type: ChangeType::Added,
                });
            }
            // Ignored
            ('!', '!') => {}
            _ => {
                // Index (staged) column
                if x != ' ' {
                    staged.push(FileChange {
                        path: path.clone(),
                        change_type: parse_change_char(x),
                    });
                }
                // Working tree (unstaged) column
                if y != ' ' {
                    unstaged.push(FileChange {
                        path,
                        change_type: parse_change_char(y),
                    });
                }
            }
        }
    }

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
    })
}

fn parse_change_char(c: char) -> ChangeType {
    match c {
        'A' => ChangeType::Added,
        'D' => ChangeType::Deleted,
        'R' => ChangeType::Renamed,
        _ => ChangeType::Modified,
    }
}

/// Get the diff of unstaged changes (or between two refs if provided).
pub fn git_diff(dir: &Path, cached: bool) -> Result<String, String> {
    if cached {
        run_git(dir, &["diff", "--cached"])
    } else {
        run_git(dir, &["diff"])
    }
}

/// Get the diff between two specific refs.
pub fn git_diff_refs(dir: &Path, base: &str, head: &str) -> Result<String, String> {
    run_git(dir, &["diff", &format!("{base}..{head}")])
}

/// Retrieve recent commit history.
pub fn git_log(dir: &Path, max_count: usize) -> Result<Vec<GitCommit>, String> {
    let sep = "---GIT_LOG_SEP---";
    let format = format!("%H%n%an%n%aI%n%s%n{sep}");
    let count_str = max_count.to_string();
    let raw = run_git(dir, &["log", &format!("--format={format}"), &format!("-{count_str}")])?;

    let commits = raw
        .split(sep)
        .filter(|chunk| !chunk.trim().is_empty())
        .filter_map(|chunk| {
            let parts: Vec<&str> = chunk.trim().split('\n').collect();
            if parts.len() >= 4 {
                let hash = parts[0].trim().to_string();
                let author = parts[1].trim().to_string();
                let date = parts[2].trim();
                let message = parts[3..].join("\n").trim().to_string();
                let date = DateTime::parse_from_rfc3339(date)
                    .ok()?
                    .with_timezone(&Utc);
                Some(GitCommit {
                    hash,
                    author,
                    date,
                    message,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

/// List all branches (local and remote).
pub fn git_branch_list(dir: &Path) -> Result<Vec<GitBranch>, String> {
    let raw = run_git(dir, &["branch", "-a", "--format=%(refname:short)\t%(HEAD)"])?;
    let branches = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            let name = parts.first().unwrap_or(&"").trim().to_string();
            let is_current = parts.get(1).map_or(false, |h| h.trim() == "*");
            let is_remote = name.starts_with("origin/");
            GitBranch {
                name,
                is_current,
                is_remote,
            }
        })
        .collect();
    Ok(branches)
}

/// Get the current branch name.
pub fn git_current_branch(dir: &Path) -> Result<String, String> {
    let raw = run_git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    Ok(raw.trim().to_string())
}

/// Stage files for commit. Pass an empty slice or `["."]` to stage all.
pub fn git_add(dir: &Path, paths: &[&str]) -> Result<(), String> {
    if paths.is_empty() {
        return run_git_ok(dir, &["add", "."]);
    }
    let mut args = vec!["add"];
    args.extend_from_slice(paths);
    run_git_ok(dir, &args)
}

/// Create a commit with the given message. Staged changes must already exist.
pub fn git_commit(dir: &Path, message: &str) -> Result<String, String> {
    let raw = run_git(dir, &["commit", "-m", message])?;
    // Extract commit hash from output
    let hash = run_git(dir, &["rev-parse", "HEAD"])?;
    Ok(hash.trim().to_string())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary directory with an initialised git repo and a first commit.
    fn setup_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create tempdir");
        let d = dir.path();
        run_git(d, &["init"]).expect("git init");
        run_git(d, &["config", "user.email", "test@test.com"]).unwrap();
        run_git(d, &["config", "user.name", "Test"]).unwrap();
        // initial commit so HEAD exists
        fs::write(d.join("README.md"), "initial\n").unwrap();
        run_git(d, &["add", "."]).unwrap();
        run_git(d, &["commit", "-m", "init"]).unwrap();
        dir
    }

    #[test]
    fn test_current_branch() {
        let dir = setup_repo();
        let branch = git_current_branch(dir.path()).unwrap();
        assert!(
            branch == "main" || branch == "master",
            "expected main or master, got {branch}"
        );
    }

    #[test]
    fn test_branch_list() {
        let dir = setup_repo();
        let branches = git_branch_list(dir.path()).unwrap();
        assert!(branches.iter().any(|b| b.is_current));
    }

    #[test]
    fn test_status_clean() {
        let dir = setup_repo();
        let status = git_status(dir.path()).unwrap();
        assert!(status.staged.is_empty());
        assert!(status.unstaged.is_empty());
        assert!(status.untracked.is_empty());
    }

    #[test]
    fn test_status_untracked_file() {
        let dir = setup_repo();
        fs::write(dir.path().join("new.txt"), "hello").unwrap();
        let status = git_status(dir.path()).unwrap();
        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_status_staged_file() {
        let dir = setup_repo();
        fs::write(dir.path().join("staged.txt"), "data").unwrap();
        git_add(dir.path(), &["staged.txt"]).unwrap();
        let status = git_status(dir.path()).unwrap();
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_diff_unstaged() {
        let dir = setup_repo();
        fs::write(dir.path().join("README.md"), "modified\n").unwrap();
        let diff = git_diff(dir.path(), false).unwrap();
        assert!(diff.contains("modified"));
    }

    #[test]
    fn test_diff_cached() {
        let dir = setup_repo();
        fs::write(dir.path().join("README.md"), "cached change\n").unwrap();
        git_add(dir.path(), &["README.md"]).unwrap();
        let diff = git_diff(dir.path(), true).unwrap();
        assert!(diff.contains("cached change"));
    }

    #[test]
    fn test_commit_and_log() {
        let dir = setup_repo();
        fs::write(dir.path().join("file.txt"), "content").unwrap();
        git_add(dir.path(), &["file.txt"]).unwrap();
        let hash = git_commit(dir.path(), "test commit").unwrap();
        assert!(!hash.is_empty());

        let log = git_log(dir.path(), 10).unwrap();
        assert!(log.len() >= 2); // init + test commit
        assert_eq!(log[0].message, "test commit");
        assert_eq!(log[0].hash, hash);
    }

    #[test]
    fn test_log_limit() {
        let dir = setup_repo();
        for i in 0..5 {
            fs::write(dir.path().join(format!("f{i}.txt")), "x").unwrap();
            git_add(dir.path(), &["."]).unwrap();
            git_commit(dir.path(), &format!("commit {i}")).unwrap();
        }
        let log = git_log(dir.path(), 3).unwrap();
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn test_diff_refs() {
        let dir = setup_repo();
        let hash1 = run_git(dir.path(), &["rev-parse", "HEAD"])
            .unwrap()
            .trim()
            .to_string();
        fs::write(dir.path().join("new_file.txt"), "new content").unwrap();
        git_add(dir.path(), &["."]).unwrap();
        git_commit(dir.path(), "second").unwrap();
        let diff = git_diff_refs(dir.path(), &hash1, "HEAD").unwrap();
        assert!(diff.contains("new content"));
    }

    #[test]
    fn test_add_multiple_files() {
        let dir = setup_repo();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        fs::write(dir.path().join("b.txt"), "b").unwrap();
        git_add(dir.path(), &["a.txt", "b.txt"]).unwrap();
        let status = git_status(dir.path()).unwrap();
        assert_eq!(status.staged.len(), 2);
    }
}
