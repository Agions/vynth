//! Per-widget dirty flags for differential rendering.

/// Per-widget dirty flags for differential rendering
#[derive(Debug, Default, Clone, Copy)]
pub struct DirtyFlags {
    pub sidebar: bool,
    pub chat: bool,
    pub diff: bool,
    pub input: bool,
    pub status: bool,
    pub approval: bool,
}

impl DirtyFlags {
    pub fn any(self) -> bool {
        self.sidebar || self.chat || self.diff || self.input || self.status || self.approval
    }

    pub fn all_dirty() -> Self {
        Self {
            sidebar: true,
            chat: true,
            diff: true,
            input: true,
            status: true,
            approval: true,
        }
    }
}
