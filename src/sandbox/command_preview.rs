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
            let path = cmd[pos + 3..]
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("?");
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
