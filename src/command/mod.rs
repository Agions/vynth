//! Command palette — fuzzy-searchable command registry + execution
//!
//! The command palette is activated via a shortcut (e.g. Ctrl+Shift+P) and
//! allows the user to search for and execute any registered command by name.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// CommandAction
// ---------------------------------------------------------------------------

/// All built-in palette actions plus a `Custom` escape hatch for plugins.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandAction {
    ToggleTheme,
    SwitchKeymap,
    RunWorkflow,
    SpawnAgent,
    GitCommit,
    GitDiff,
    OpenFile,
    ToggleSidebar,
    ToggleDiff,
    ChangeSandboxMode,
    ReloadConfig,
    Quit,
    /// Arbitrary plugin / extension action identified by name.
    Custom(String),
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// A single executable command that can appear in the command palette.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub category: String,
    pub shortcut: Option<String>,
    pub action: CommandAction,
}

impl Command {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        action: CommandAction,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: category.into(),
            shortcut: None,
            action,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

// ---------------------------------------------------------------------------
// CommandPalette
// ---------------------------------------------------------------------------

/// An interactive, fuzzy-filterable command palette.
#[derive(Debug)]
pub struct CommandPalette {
    pub commands: Vec<Command>,
    pub query: String,
    pub selected_index: usize,
    pub filtered: Vec<usize>,
    pub visible: bool,
}

impl CommandPalette {
    /// Create a palette pre-loaded with the given commands.
    pub fn new(commands: Vec<Command>) -> Self {
        let filtered = (0..commands.len()).collect();
        Self {
            commands,
            query: String::new(),
            selected_index: 0,
            filtered,
            visible: false,
        }
    }

    // -- visibility ----------------------------------------------------------

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.rebuild_filtered();
        self.selected_index = 0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    // -- query ---------------------------------------------------------------

    /// Update the fuzzy query and rebuild the filtered list.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.rebuild_filtered();
        // Clamp selected_index
        if self.selected_index >= self.filtered.len() {
            self.selected_index = self.filtered.len().saturating_sub(1);
        }
    }

    /// Return a score for how well `haystack` matches `query`.
    ///
    /// Scoring strategy (higher is better):
    ///   +100  exact substring match (case-insensitive)
    ///   +10   per matched character in order (subsequence)
    ///    -1   penalty per gap between matched chars
    ///
    /// Returns `None` when no subsequence match is found.
    pub fn fuzzy_match(query: &str, haystack: &str) -> Option<i64> {
        let q = query.to_lowercase();
        let h = haystack.to_lowercase();

        // Fast path: exact substring bonus
        if h.contains(&q) {
            return Some(100 + q.len() as i64 * 10);
        }

        // Subsequence matching
        let q_chars: Vec<char> = q.chars().collect();
        let h_chars: Vec<char> = h.chars().collect();

        if q_chars.is_empty() {
            return Some(0);
        }

        let mut qi = 0usize;
        let mut score: i64 = 0;
        let mut last_match_pos: Option<usize> = None;

        for (hi, &hc) in h_chars.iter().enumerate() {
            if hc == q_chars[qi] {
                score += 10;
                // Penalise gaps
                if let Some(prev) = last_match_pos {
                    let gap = hi.saturating_sub(prev + 1) as i64;
                    score -= gap;
                }
                last_match_pos = Some(hi);
                qi += 1;
                if qi == q_chars.len() {
                    break;
                }
            }
        }

        if qi == q_chars.len() {
            Some(score)
        } else {
            None
        }
    }

    /// Rebuild `self.filtered` based on current query.
    fn rebuild_filtered(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commands.len()).collect();
            return;
        }

        let mut scored: Vec<(usize, i64)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                // Match against name or description
                let name_score = Self::fuzzy_match(&self.query, &cmd.name);
                let desc_score = Self::fuzzy_match(&self.query, &cmd.description);
                let best = match (name_score, desc_score) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
                best.map(|s| (i, s))
            })
            .collect();

        // Sort by score descending, then by name for stability
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| self.commands[a.0].name.cmp(&self.commands[b.0].name)));

        self.filtered = scored.into_iter().map(|(i, _)| i).collect();
    }

    // -- navigation ----------------------------------------------------------

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index + 1 < self.filtered.len() {
            self.selected_index += 1;
        }
    }

    // -- selection / execution -----------------------------------------------

    /// Return a reference to the currently selected command, if any.
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered
            .get(self.selected_index)
            .and_then(|&idx| self.commands.get(idx))
    }

    /// Return the action of the currently selected command.
    pub fn execute_selected(&self) -> Option<CommandAction> {
        self.selected_command().map(|cmd| cmd.action.clone())
    }
}

// ---------------------------------------------------------------------------
// Default command set
// ---------------------------------------------------------------------------

/// Build the default palette commands shipped with Syncode.
pub fn default_commands() -> Vec<Command> {
    vec![
        Command::new("Toggle Theme", "Switch between light and dark theme", "View", CommandAction::ToggleTheme)
            .with_shortcut("Ctrl+T"),
        Command::new("Switch Keymap", "Switch keyboard shortcut profile", "Settings", CommandAction::SwitchKeymap),
        Command::new("Run Workflow", "Execute a named workflow", "Tools", CommandAction::RunWorkflow),
        Command::new("Spawn Agent", "Spawn a new agent subtask", "Tools", CommandAction::SpawnAgent),
        Command::new("Git Commit", "Create a git commit", "Git", CommandAction::GitCommit),
        Command::new("Git Diff", "Show git diff", "Git", CommandAction::GitDiff)
            .with_shortcut("Ctrl+D"),
        Command::new("Open File", "Open a file in the editor", "File", CommandAction::OpenFile)
            .with_shortcut("Ctrl+O"),
        Command::new("Toggle Sidebar", "Show or hide the sidebar panel", "View", CommandAction::ToggleSidebar)
            .with_shortcut("Ctrl+B"),
        Command::new("Toggle Diff", "Show or hide the diff panel", "View", CommandAction::ToggleDiff),
        Command::new("Change Sandbox Mode", "Switch sandbox execution mode", "Settings", CommandAction::ChangeSandboxMode),
        Command::new("Reload Config", "Reload configuration from disk", "Settings", CommandAction::ReloadConfig)
            .with_shortcut("Ctrl+Shift+R"),
        Command::new("Quit", "Exit Syncode", "Application", CommandAction::Quit)
            .with_shortcut("Ctrl+Q"),
        Command::new(
            "Code Review",
            "Run the code review skill",
            "Skills",
            CommandAction::Custom("code_review".into()),
        ),
        Command::new(
            "Refactor",
            "Run the refactor skill on selection",
            "Skills",
            CommandAction::Custom("refactor".into()),
        ),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_commands() -> Vec<Command> {
        default_commands()
    }

    #[test]
    fn test_palette_creation() {
        let palette = CommandPalette::new(sample_commands());
        assert!(!palette.visible);
        assert_eq!(palette.query, "");
        assert_eq!(palette.selected_index, 0);
        assert_eq!(palette.filtered.len(), palette.commands.len());
    }

    #[test]
    fn test_show_hide_toggle() {
        let mut palette = CommandPalette::new(sample_commands());
        assert!(!palette.visible);

        palette.show();
        assert!(palette.visible);
        assert_eq!(palette.query, "");

        palette.hide();
        assert!(!palette.visible);

        palette.toggle();
        assert!(palette.visible);

        palette.toggle();
        assert!(!palette.visible);
    }

    #[test]
    fn test_set_query_filters_results() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.show();

        palette.set_query("theme");
        assert!(palette.filtered.len() < palette.commands.len());
        // The first filtered command should be "Toggle Theme"
        let cmd = palette.selected_command().unwrap();
        assert_eq!(cmd.name, "Toggle Theme");
    }

    #[test]
    fn test_set_query_empty_shows_all() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.set_query("git");
        assert!(palette.filtered.len() < palette.commands.len());

        palette.set_query("");
        assert_eq!(palette.filtered.len(), palette.commands.len());
    }

    #[test]
    fn test_fuzzy_match_exact_substring() {
        let score = CommandPalette::fuzzy_match("theme", "Toggle Theme").unwrap();
        assert!(score > 0);
    }

    #[test]
    fn test_fuzzy_match_subsequence() {
        let score = CommandPalette::fuzzy_match("tgl", "Toggle");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let score = CommandPalette::fuzzy_match("xyz", "Toggle Theme");
        assert!(score.is_none());
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let score = CommandPalette::fuzzy_match("THEME", "Toggle Theme");
        assert!(score.is_some());
    }

    #[test]
    fn test_navigate_up_down() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.show();
        assert_eq!(palette.selected_index, 0);

        palette.navigate_down();
        assert_eq!(palette.selected_index, 1);

        palette.navigate_down();
        assert_eq!(palette.selected_index, 2);

        palette.navigate_up();
        assert_eq!(palette.selected_index, 1);

        // Cannot go below 0
        palette.selected_index = 0;
        palette.navigate_up();
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_navigate_clamped_at_end() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.show();
        let last = palette.commands.len() - 1;

        for _ in 0..last + 5 {
            palette.navigate_down();
        }
        assert_eq!(palette.selected_index, last);
    }

    #[test]
    fn test_execute_selected_returns_action() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.show();
        palette.set_query("Quit");
        let action = palette.execute_selected();
        assert_eq!(action, Some(CommandAction::Quit));
    }

    #[test]
    fn test_execute_selected_none_when_empty() {
        let mut palette = CommandPalette::new(vec![]);
        palette.show();
        assert!(palette.execute_selected().is_none());
    }

    #[test]
    fn test_query_resets_selected_index_on_shrink() {
        let mut palette = CommandPalette::new(sample_commands());
        palette.show();
        palette.set_query("git");
        // navigate to last
        palette.selected_index = palette.filtered.len();
        palette.navigate_up(); // now at valid last
        let idx_before = palette.selected_index;

        // Narrow query to only one result
        palette.set_query("Git Commit");
        assert!(palette.selected_index < palette.filtered.len());
    }

    #[test]
    fn test_command_with_shortcut() {
        let cmd = Command::new("Test", "desc", "cat", CommandAction::Quit)
            .with_shortcut("Ctrl+Q");
        assert_eq!(cmd.shortcut, Some("Ctrl+Q".into()));
    }

    #[test]
    fn test_default_commands_non_empty() {
        let cmds = default_commands();
        assert!(cmds.len() >= 10);
    }
}
