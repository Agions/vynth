//! Shared utility functions

use std::path::PathBuf;

/// Recursively walk a directory for files
pub async fn walk_dir(path: &std::path::Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let mut read_dir = match tokio::fs::read_dir(path).await {
        Ok(rd) => rd,
        Err(_) => return results,
    };
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            results.extend(Box::pin(walk_dir(&path)).await);
        } else {
            results.push(path);
        }
    }
    results
}
