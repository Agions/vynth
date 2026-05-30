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
