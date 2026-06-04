//! Structured audit logging for sandbox operations.
// TODO: Audit — awaiting sandbox integration
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use std::sync::Mutex;

/// Risk level of an audited command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditRisk {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

/// Approval mode used for execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditApproval {
    Auto,
    Confirmed,
    PreviewOnly,
    Denied,
}

/// Single audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub risk_level: AuditRisk,
    pub approval: AuditApproval,
    pub success: bool,
    pub session_id: String,
}

/// In-memory audit log (append-only, bounded to 10k entries)
pub struct AuditLog {
    entries: Mutex<Vec<AuditEntry>>,
    max_entries: usize,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            max_entries: 10_000,
        }
    }

    pub fn record(&self, entry: AuditEntry) {
        let mut entries = self.entries.lock().expect("audit log poisoned");
        if entries.len() >= self.max_entries {
            entries.remove(0); // FIFO eviction oldest entry
        }
        entries.push(entry);
    }

    pub fn recent(&self, count: usize) -> Vec<AuditEntry> {
        let entries = self.entries.lock().expect("audit log poisoned");
        entries.iter().rev().take(count).cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}