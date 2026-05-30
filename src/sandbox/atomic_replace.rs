//! Atomic file replacement — write-then-rename for crash safety

use std::path::Path;

use crate::error::AppError;

/// Atomically write content to a file (write temp → fsync → rename)
///
/// This ensures that either the old file or the new file exists —
/// no partial writes or corruption on crash.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), AppError> {
    let tmp_path = path.with_extension("syncode.tmp");

    // 1. Write to temp file
    std::fs::write(&tmp_path, content)?;

    // 2. Fsync (ensure data is on disk)
    #[cfg(unix)]
    {
        let file = std::fs::OpenOptions::new().write(true).open(&tmp_path)?;
        file.sync_all()?;
    }

    // 3. Atomic rename
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

/// Atomic write with backup — preserves the old file as .bak
pub fn atomic_write_with_backup(path: &Path, content: &[u8]) -> Result<(), AppError> {
    // Create backup of existing file
    if path.exists() {
        let bak_path = path.with_extension("syncode.bak");
        std::fs::copy(path, &bak_path)?;
    }

    atomic_write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_atomic_write() {
        let dir = std::env::temp_dir().join("syncode_test_atomic");
        let _ = fs::create_dir_all(&dir);

        let path = dir.join("test.txt");

        // Write
        atomic_write(&path, b"hello world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");

        // Overwrite
        atomic_write(&path, b"goodbye world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "goodbye world");

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
