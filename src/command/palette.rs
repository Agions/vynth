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
