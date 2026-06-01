use super::{Command, CommandAction};

/// Build the default palette commands shipped with Synerix.
pub fn default_commands() -> Vec<Command> {
    vec![
        Command::new(
            "Toggle Theme",
            "Switch between light and dark theme",
            "View",
            CommandAction::ToggleTheme,
        )
        .with_shortcut("Ctrl+T"),
        Command::new(
            "Switch Keymap",
            "Switch keyboard shortcut profile",
            "Settings",
            CommandAction::SwitchKeymap,
        ),
        Command::new(
            "Run Workflow",
            "Execute a named workflow",
            "Tools",
            CommandAction::RunWorkflow,
        ),
        Command::new(
            "Spawn Agent",
            "Spawn a new agent subtask",
            "Tools",
            CommandAction::SpawnAgent,
        ),
        Command::new(
            "Git Commit",
            "Create a git commit",
            "Git",
            CommandAction::GitCommit,
        ),
        Command::new("Git Diff", "Show git diff", "Git", CommandAction::GitDiff)
            .with_shortcut("Ctrl+D"),
        Command::new(
            "Open File",
            "Open a file in the editor",
            "File",
            CommandAction::OpenFile,
        )
        .with_shortcut("Ctrl+O"),
        Command::new(
            "Toggle Sidebar",
            "Show or hide the sidebar panel",
            "View",
            CommandAction::ToggleSidebar,
        )
        .with_shortcut("Ctrl+B"),
        Command::new(
            "Toggle Diff",
            "Show or hide the diff panel",
            "View",
            CommandAction::ToggleDiff,
        ),
        Command::new(
            "Change Sandbox Mode",
            "Switch sandbox execution mode",
            "Settings",
            CommandAction::ChangeSandboxMode,
        ),
        Command::new(
            "Reload Config",
            "Reload configuration from disk",
            "Settings",
            CommandAction::ReloadConfig,
        )
        .with_shortcut("Ctrl+Shift+R"),
        Command::new("Quit", "Exit Synerix", "Application", CommandAction::Quit)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_commands_count() {
        let cmds = default_commands();
        assert_eq!(cmds.len(), 14);
    }

    #[test]
    fn test_default_commands_have_names() {
        let cmds = default_commands();
        for cmd in &cmds {
            assert!(!cmd.name.is_empty(), "Command name should not be empty");
            assert!(
                !cmd.description.is_empty(),
                "Command description should not be empty"
            );
            assert!(
                !cmd.category.is_empty(),
                "Command category should not be empty"
            );
        }
    }

    #[test]
    fn test_specific_commands_exist() {
        let cmds = default_commands();
        let names: Vec<&str> = cmds.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Toggle Theme"));
        assert!(names.contains(&"Quit"));
        assert!(names.contains(&"Git Commit"));
        assert!(names.contains(&"Git Diff"));
        assert!(names.contains(&"Open File"));
        assert!(names.contains(&"Toggle Sidebar"));
        assert!(names.contains(&"Reload Config"));
    }

    #[test]
    fn test_shortcuts_assigned() {
        let cmds = default_commands();
        let toggle_theme = cmds.iter().find(|c| c.name == "Toggle Theme").unwrap();
        assert_eq!(toggle_theme.shortcut, Some("Ctrl+T".into()));

        let git_diff = cmds.iter().find(|c| c.name == "Git Diff").unwrap();
        assert_eq!(git_diff.shortcut, Some("Ctrl+D".into()));

        let open_file = cmds.iter().find(|c| c.name == "Open File").unwrap();
        assert_eq!(open_file.shortcut, Some("Ctrl+O".into()));

        let quit = cmds.iter().find(|c| c.name == "Quit").unwrap();
        assert_eq!(quit.shortcut, Some("Ctrl+Q".into()));
    }

    #[test]
    fn test_categories() {
        let cmds = default_commands();
        let categories: Vec<&str> = cmds.iter().map(|c| c.category.as_str()).collect();
        assert!(categories.contains(&"View"));
        assert!(categories.contains(&"Settings"));
        assert!(categories.contains(&"Tools"));
        assert!(categories.contains(&"Git"));
        assert!(categories.contains(&"File"));
        assert!(categories.contains(&"Application"));
        assert!(categories.contains(&"Skills"));
    }

    #[test]
    fn test_skill_commands_use_custom_action() {
        let cmds = default_commands();
        let code_review = cmds.iter().find(|c| c.name == "Code Review").unwrap();
        assert!(matches!(
            &code_review.action,
            CommandAction::Custom(name) if name == "code_review"
        ));

        let refactor = cmds.iter().find(|c| c.name == "Refactor").unwrap();
        assert!(matches!(
            &refactor.action,
            CommandAction::Custom(name) if name == "refactor"
        ));
    }
}
