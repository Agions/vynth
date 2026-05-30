//! Configuration file hot-reload watcher
//!
//! Uses a simple polling approach: every 2 seconds, check the mtime of
//! the config file. If it has changed, reload settings and send them
//! through a channel.
//!
//! On Unix, SIGHUP is also handled as an explicit reload signal.

use std::path::PathBuf;
use std::time::SystemTime;
use tokio::sync::mpsc;

use crate::config::Settings;

/// Message sent when config is reloaded
pub struct ConfigReload {
    pub settings: Settings,
    pub version: u64,
}

/// Spawn the config file watcher as a background task.
///
/// Returns a receiver that yields [`ConfigReload`] messages whenever the
/// config file is modified or SIGHUP is received (Unix only).
pub fn spawn_config_watcher(
    config_path: PathBuf,
    initial_version: u64,
) -> mpsc::UnboundedReceiver<ConfigReload> {
    let (tx, rx) = mpsc::unbounded_channel();

    let tx_poll = tx.clone();
    let path_poll = config_path.clone();

    // Polling watcher
    tokio::spawn(async move {
        watch_loop(path_poll, initial_version, tx_poll).await;
    });

    // SIGHUP handler (Unix only)
    #[cfg(unix)]
    {
        tokio::spawn(async move {
            handle_sighup(config_path, tx).await;
        });
    }

    rx
}

/// Polling loop: check mtime every 2 seconds
async fn watch_loop(
    config_path: PathBuf,
    mut version: u64,
    tx: mpsc::UnboundedSender<ConfigReload>,
) {
    let mut last_mtime = get_mtime(&config_path);
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

    // Skip the first immediate tick
    interval.tick().await;

    loop {
        interval.tick().await;

        let current_mtime = get_mtime(&config_path);
        if current_mtime != last_mtime {
            last_mtime = current_mtime;
            version += 1;

            match Settings::load() {
                Ok(settings) => {
                    tracing::info!(
                        version = version,
                        "Config file changed, reloaded settings"
                    );
                    let _ = tx.send(ConfigReload { settings, version });
                }
                Err(e) => {
                    tracing::error!("Config reload failed: {}, keeping previous settings", e);
                }
            }
        }
    }
}

/// Get the modification time of a file, returning None if it doesn't exist
fn get_mtime(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
}

/// Handle SIGHUP signal on Unix — reload config on receipt
#[cfg(unix)]
async fn handle_sighup(
    config_path: PathBuf,
    tx: mpsc::UnboundedSender<ConfigReload>,
) {
    use tokio::signal::unix::{signal, SignalKind};

    let mut stream = match signal(SignalKind::hangup()) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to register SIGHUP handler: {}", e);
            return;
        }
    };

    let mut version: u64 = 0;
    loop {
        stream.recv().await;
        version += 1;
        tracing::info!("SIGHUP received, reloading config (version={})", version);

        match Settings::load() {
            Ok(settings) => {
                let _ = tx.send(ConfigReload { settings, version });
            }
            Err(e) => {
                tracing::error!("Config reload on SIGHUP failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_mtime_returns_none_for_missing_file() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        assert!(get_mtime(&path).is_none());
    }
}
