//! Command preview — analyze and display command impact before execution

/// Analysis of a shell command's potential impact
pub struct CommandPreview {
    pub command: String,
    pub risk_level: RiskLevel,
    pub description: String,
    pub affected_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl CommandPreview {
    /// Analyze a command and produce a preview
    pub fn analyze(command: &str) -> Self {
        let (risk, desc, paths) = classify_command(command);

        Self {
            command: command.to_string(),
            risk_level: risk,
            description: desc,
            affected_paths: paths,
        }
    }

    /// Format preview for user display
    pub fn display(&self) -> String {
        let risk_emoji = match self.risk_level {
            RiskLevel::Safe => "✅",
            RiskLevel::Low => "ℹ️",
            RiskLevel::Medium => "⚠️",
            RiskLevel::High => "🔶",
            RiskLevel::Critical => "🔴",
        };

        format!(
            "{} [{}] {}\n   Command: {}\n   Affected: {}",
            risk_emoji,
            format!("{:?}", self.risk_level).to_uppercase(),
            self.description,
            self.command,
            if self.affected_paths.is_empty() {
                "none detected".to_string()
            } else {
                self.affected_paths.join(", ")
            }
        )
    }
}

/// Classify a command by risk level
fn classify_command(cmd: &str) -> (RiskLevel, String, Vec<String>) {
    let cmd_lower = cmd.to_lowercase();
    let mut paths = Vec::new();

    // Critical: destructive operations
    if cmd_lower.contains("rm -rf") || cmd_lower.contains("rm -r /") {
        paths.push("/ (recursive delete)".to_string());
        return (
            RiskLevel::Critical,
            "Recursive delete — will permanently remove files".to_string(),
            paths,
        );
    }

    if cmd_lower.contains("mkfs") || cmd_lower.contains("dd if=") {
        return (
            RiskLevel::Critical,
            "Disk operation — may overwrite data".to_string(),
            paths,
        );
    }

    // High: system modifications
    if cmd_lower.contains("chmod -r") || cmd_lower.contains("chown -r") {
        return (
            RiskLevel::High,
            "Recursive permission change".to_string(),
            paths,
        );
    }

    if cmd_lower.contains("sudo ") {
        return (
            RiskLevel::High,
            "Elevated privilege command".to_string(),
            paths,
        );
    }

    // Medium: file writes
    if cmd.contains(" > ") || cmd.contains(" >> ") {
        if let Some(pos) = cmd.find(" > ") {
            let path = cmd[pos + 3..].split_whitespace().next().unwrap_or("?");
            paths.push(path.to_string());
        }
        return (
            RiskLevel::Medium,
            "File write/redirection".to_string(),
            paths,
        );
    }

    // Low: git operations
    if cmd_lower.starts_with("git ") {
        if cmd_lower.contains("push") || cmd_lower.contains("force") {
            return (RiskLevel::Medium, "Git push operation".to_string(), paths);
        }
        return (RiskLevel::Low, "Git operation".to_string(), paths);
    }

    // Safe: read-only commands
    if cmd_lower.starts_with("ls")
        || cmd_lower.starts_with("cat")
        || cmd_lower.starts_with("echo")
        || cmd_lower.starts_with("pwd")
        || cmd_lower.starts_with("which")
        || cmd_lower.starts_with("grep")
        || cmd_lower.starts_with("find")
    {
        return (RiskLevel::Safe, "Read-only operation".to_string(), paths);
    }

    (RiskLevel::Low, "General command".to_string(), paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Critical risk ──────────────────────────────────────

    #[test]
    fn test_critical_rm_rf() {
        let preview = CommandPreview::analyze("rm -rf /tmp/data");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
        assert!(preview.description.contains("Recursive delete"));
        assert!(!preview.affected_paths.is_empty());
    }

    #[test]
    fn test_critical_rm_r_root() {
        let preview = CommandPreview::analyze("rm -r /");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_critical_dd() {
        let preview = CommandPreview::analyze("dd if=/dev/zero of=/dev/sda");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
    }

    // ── High risk ──────────────────────────────────────────

    #[test]
    fn test_high_sudo() {
        let preview = CommandPreview::analyze("sudo apt install nginx");
        assert_eq!(preview.risk_level, RiskLevel::High);
        assert!(preview.description.contains("privilege"));
    }

    #[test]
    fn test_high_chmod_r() {
        let preview = CommandPreview::analyze("chmod -R 777 /var/www");
        assert_eq!(preview.risk_level, RiskLevel::High);
        assert!(preview.description.contains("permission"));
    }

    #[test]
    fn test_high_chown_r() {
        let preview = CommandPreview::analyze("chown -R user:group /home");
        assert_eq!(preview.risk_level, RiskLevel::High);
    }

    // ── Medium risk ────────────────────────────────────────

    #[test]
    fn test_medium_redirect() {
        let preview = CommandPreview::analyze("echo hello > output.txt");
        assert_eq!(preview.risk_level, RiskLevel::Medium);
        assert!(preview.description.contains("write"));
        assert!(preview.affected_paths.contains(&"output.txt".to_string()));
    }

    #[test]
    fn test_medium_append() {
        let preview = CommandPreview::analyze("echo data >> log.txt");
        assert_eq!(preview.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_medium_git_push() {
        let preview = CommandPreview::analyze("git push origin main");
        assert_eq!(preview.risk_level, RiskLevel::Medium);
        assert!(preview.description.contains("push"));
    }

    #[test]
    fn test_medium_git_force() {
        let preview = CommandPreview::analyze("git push --force");
        assert_eq!(preview.risk_level, RiskLevel::Medium);
    }

    // ── Low risk ───────────────────────────────────────────

    #[test]
    fn test_low_git_status() {
        let preview = CommandPreview::analyze("git status");
        assert_eq!(preview.risk_level, RiskLevel::Low);
        assert!(preview.description.contains("Git"));
    }

    #[test]
    fn test_low_git_diff() {
        let preview = CommandPreview::analyze("git diff");
        assert_eq!(preview.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_low_general_command() {
        let preview = CommandPreview::analyze("cargo build");
        assert_eq!(preview.risk_level, RiskLevel::Low);
        assert!(preview.description.contains("General"));
    }

    // ── Safe commands ──────────────────────────────────────

    #[test]
    fn test_safe_ls() {
        let preview = CommandPreview::analyze("ls -la");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
        assert!(preview.description.contains("Read-only"));
    }

    #[test]
    fn test_safe_cat() {
        let preview = CommandPreview::analyze("cat /etc/hosts");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_safe_echo() {
        let preview = CommandPreview::analyze("echo hello");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_safe_pwd() {
        let preview = CommandPreview::analyze("pwd");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_safe_grep() {
        let preview = CommandPreview::analyze("grep -r 'pattern' .");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_safe_find() {
        let preview = CommandPreview::analyze("find . -name '*.rs'");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    // ── Display formatting ─────────────────────────────────

    #[test]
    fn test_display_contains_emoji() {
        let preview = CommandPreview::analyze("ls");
        let display = preview.display();
        assert!(display.contains("✅"));
        assert!(display.contains("SAFE"));
    }

    #[test]
    fn test_display_critical_emoji() {
        let preview = CommandPreview::analyze("rm -rf /");
        let display = preview.display();
        assert!(display.contains("🔴"));
        assert!(display.contains("CRITICAL"));
    }

    #[test]
    fn test_display_no_paths() {
        let preview = CommandPreview::analyze("ls");
        let display = preview.display();
        assert!(display.contains("none detected"));
    }

    #[test]
    fn test_display_with_paths() {
        let preview = CommandPreview::analyze("echo data > output.txt");
        let display = preview.display();
        assert!(display.contains("output.txt"));
    }

    // ── Risk level ordering ────────────────────────────────

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }
}
