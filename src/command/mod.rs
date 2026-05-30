//! Command palette — fuzzy-searchable command registry + execution
//!
//! The command palette is activated via a shortcut (e.g. Ctrl+Shift+P) and
//! allows the user to search for and execute any registered command by name.

mod action;
mod command;
mod defaults;
mod palette;

pub use action::CommandAction;
pub use command::Command;
pub use defaults::default_commands;
pub use palette::CommandPalette;

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
        let cmd = Command::new("Test", "desc", "cat", CommandAction::Quit).with_shortcut("Ctrl+Q");
        assert_eq!(cmd.shortcut, Some("Ctrl+Q".into()));
    }

    #[test]
    fn test_default_commands_non_empty() {
        let cmds = default_commands();
        assert!(cmds.len() >= 10);
    }
}
