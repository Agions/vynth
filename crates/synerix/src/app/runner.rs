//! Application runner — entry point for running the app

use super::state::App;
use crate::agent::CustomAgentRegistry;
use crate::error::AppError;
use crate::skills::SkillRegistry;

/// Run the application
pub async fn run(
    settings: crate::config::Settings,
    startup_metrics: crate::telemetry::StartupMetrics,
) -> Result<(), AppError> {
    tracing::info!("Initializing application");

    // Spawn config file watcher (polls mtime + SIGHUP on unix)
    let config_path = dirs_next::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("synerix")
        .join("config.toml");
    let config_reload_rx = crate::config::spawn_config_watcher(config_path, 0);

    // Create app with the config reload channel wired in
    let mut app = App::new_with_settings(settings, config_reload_rx);

    // Load project-local skills and agents from `.synerix/` directory
    load_synerix_resources(&mut app).await;

    // Attach startup metrics to status bar
    app.status_bar.startup_metrics = Some(startup_metrics);

    // Initialize TUI
    let mut terminal = crate::tui::init()?;

    // Initialize theme (dark by default; TODO: read from config)
    crate::tui::theme::init_theme(true);

    let result = app.run(&mut terminal).await;

    // Restore terminal
    crate::tui::restore(terminal)?;

    result
}

/// Load skills from `.synerix/skills/` and agents from `.synerix/agents/`
///
/// Follows the same pattern as Claude Code's `.claude/skills/` and `.claude/agents/`.
/// Both loads run in parallel via `tokio::join!` to reduce startup time.
async fn load_synerix_resources(app: &mut App) {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Cannot determine current directory: {e}");
            return;
        }
    };

    // Run both I/O-bound loads in parallel
    let (skills_result, agents_result) = tokio::join!(load_skills(&cwd), load_agents(&cwd),);

    if let Some(registry) = skills_result {
        let count = registry.list_names().len();
        app.skill_registry = registry;
        tracing::info!("Loaded {count} skills from .synerix/skills/");
    } else {
        tracing::debug!(".synerix/skills/ not found, skipping");
    }

    if let Some(registry) = agents_result {
        let count = registry.len();
        app.agent_registry = registry;
        tracing::info!("Loaded {count} agents from .synerix/agents/");
    } else {
        tracing::debug!(".synerix/agents/ not found, skipping");
    }
}

/// Load skills from `.synerix/skills/` directory.
async fn load_skills(cwd: &std::path::Path) -> Option<SkillRegistry> {
    let skills_dir = crate::project::detector::find_synerix_skills_dir(cwd)?;
    match SkillRegistry::load_from_dir(&skills_dir).await {
        Ok(registry) => Some(registry),
        Err(e) => {
            tracing::warn!("Failed to load skills from .synerix/skills/: {e}");
            None
        }
    }
}

/// Load custom agents from `.synerix/agents/` directory.
async fn load_agents(cwd: &std::path::Path) -> Option<CustomAgentRegistry> {
    let agents_dir = crate::project::detector::find_synerix_agents_dir(cwd)?;
    match CustomAgentRegistry::load_from_dir(&agents_dir).await {
        Ok(registry) => Some(registry),
        Err(e) => {
            tracing::warn!("Failed to load agents from .synerix/agents/: {e}");
            None
        }
    }
}
