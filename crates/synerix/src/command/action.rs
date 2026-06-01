//! Command action types — the set of all operations the palette can trigger.

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
