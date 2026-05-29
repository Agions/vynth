//! Keymap profiles

use serde::{Deserialize, Serialize};

/// Keymap profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeymapProfile {
    Vim,
    Emacs,
    Default,
}

impl Default for KeymapProfile {
    fn default() -> Self {
        Self::Default
    }
}
