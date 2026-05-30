use super::{Command, CommandAction};

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
        scored.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| self.commands[a.0].name.cmp(&self.commands[b.0].name))
        });

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandAction;

    fn test_commands() -> Vec<Command> {
        vec![
            Command::new(
                "Open File",
                "Open a file in the editor",
                "File",
                CommandAction::OpenFile,
            ),
            Command::new(
                "Save File",
                "Save the current file",
                "File",
                CommandAction::OpenFile,
            ),
            Command::new(
                "Close Tab",
                "Close the current tab",
                "Tabs",
                CommandAction::OpenFile,
            ),
            Command::new(
                "Toggle Sidebar",
                "Show or hide the sidebar",
                "View",
                CommandAction::OpenFile,
            ),
            Command::new(
                "Run Tests",
                "Run the test suite",
                "Build",
                CommandAction::OpenFile,
            ),
        ]
    }

    // ── fuzzy_match ────────────────────────────────────────

    #[test]
    fn test_fuzzy_exact_substring() {
        let score = CommandPalette::fuzzy_match("save", "Save File");
        assert!(score.is_some());
        assert!(score.unwrap() > 100);
    }

    #[test]
    fn test_fuzzy_subsequence() {
        let score = CommandPalette::fuzzy_match("sf", "Save File");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_no_match() {
        let score = CommandPalette::fuzzy_match("xyz", "Save File");
        assert!(score.is_none());
    }

    #[test]
    fn test_fuzzy_empty_query() {
        let score = CommandPalette::fuzzy_match("", "anything");
        // Empty query matches everything via the substring fast path
        assert!(score.is_some());
        assert!(score.unwrap() >= 0);
    }

    #[test]
    fn test_fuzzy_case_insensitive() {
        let score = CommandPalette::fuzzy_match("SAVE", "save file");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_exact_beats_subsequence() {
        let exact = CommandPalette::fuzzy_match("save", "Save File").unwrap();
        let subseq = CommandPalette::fuzzy_match("sf", "Save File").unwrap();
        assert!(exact > subseq);
    }

    // ── CommandPalette lifecycle ───────────────────────────

    #[test]
    fn test_palette_new() {
        let palette = CommandPalette::new(test_commands());
        assert_eq!(palette.commands.len(), 5);
        assert_eq!(palette.filtered.len(), 5);
        assert!(!palette.visible);
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_palette_show_hide() {
        let mut palette = CommandPalette::new(test_commands());
        palette.show();
        assert!(palette.visible);
        assert_eq!(palette.selected_index, 0);

        palette.hide();
        assert!(!palette.visible);
    }

    #[test]
    fn test_palette_toggle() {
        let mut palette = CommandPalette::new(test_commands());
        palette.toggle();
        assert!(palette.visible);
        palette.toggle();
        assert!(!palette.visible);
    }

    // ── Query filtering ────────────────────────────────────

    #[test]
    fn test_set_query_filters() {
        let mut palette = CommandPalette::new(test_commands());
        palette.set_query("save");
        assert!(palette.filtered.len() < 5);
        // "Save File" should be in filtered results
        let names: Vec<&str> = palette
            .filtered
            .iter()
            .map(|&i| palette.commands[i].name.as_str())
            .collect();
        assert!(names.contains(&"Save File"));
    }

    #[test]
    fn test_empty_query_shows_all() {
        let mut palette = CommandPalette::new(test_commands());
        palette.set_query("save");
        palette.set_query("");
        assert_eq!(palette.filtered.len(), 5);
    }

    #[test]
    fn test_query_no_match() {
        let mut palette = CommandPalette::new(test_commands());
        palette.set_query("zzzzz");
        assert!(palette.filtered.is_empty());
    }

    // ── Navigation ─────────────────────────────────────────

    #[test]
    fn test_navigate_down() {
        let mut palette = CommandPalette::new(test_commands());
        assert_eq!(palette.selected_index, 0);
        palette.navigate_down();
        assert_eq!(palette.selected_index, 1);
    }

    #[test]
    fn test_navigate_down_at_end() {
        let mut palette = CommandPalette::new(test_commands());
        for _ in 0..10 {
            palette.navigate_down();
        }
        assert_eq!(palette.selected_index, 4); // clamped to last
    }

    #[test]
    fn test_navigate_up() {
        let mut palette = CommandPalette::new(test_commands());
        palette.navigate_down();
        palette.navigate_down();
        palette.navigate_up();
        assert_eq!(palette.selected_index, 1);
    }

    #[test]
    fn test_navigate_up_at_start() {
        let mut palette = CommandPalette::new(test_commands());
        palette.navigate_up(); // should not underflow
        assert_eq!(palette.selected_index, 0);
    }

    // ── Selection ──────────────────────────────────────────

    #[test]
    fn test_selected_command() {
        let palette = CommandPalette::new(test_commands());
        let cmd = palette.selected_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "Open File");
    }

    #[test]
    fn test_execute_selected() {
        let palette = CommandPalette::new(test_commands());
        let action = palette.execute_selected();
        assert!(action.is_some());
    }

    #[test]
    fn test_selected_after_filter() {
        let mut palette = CommandPalette::new(test_commands());
        palette.set_query("save");
        let cmd = palette.selected_command();
        assert!(cmd.is_some());
    }
}
