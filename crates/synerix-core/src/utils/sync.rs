//! Synchronisation primitives — extensions over `std::sync`

use std::sync::{Mutex, MutexGuard};

/// Extension trait for `std::sync::Mutex` providing ergonomic error conversion.
pub trait MutexExt<T> {
    /// Lock the mutex, converting a `PoisonError` into a descriptive `String` error.
    fn lock_or_err(&self) -> Result<MutexGuard<'_, T>, String>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_or_err(&self) -> Result<MutexGuard<'_, T>, String> {
        self.lock().map_err(|e| e.to_string())
    }
}