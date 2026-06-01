//! Command definition — a single executable item in the command palette.

use super::CommandAction;

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
