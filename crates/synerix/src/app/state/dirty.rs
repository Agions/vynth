//! Per-widget dirty flags for differential rendering.

use bitflags::bitflags;

bitflags! {
    /// Per-widget dirty flags for differential rendering.
    ///
    /// Rendering is skipped for widgets whose flag is not set, saving CPU
    /// when only a subset of the UI changed (e.g. a tick that only needs
    /// to redraw the status bar).
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct DirtyFlags: u8 {
        const SIDEBAR = 1 << 0;
        const CHAT     = 1 << 1;
        const DIFF     = 1 << 2;
        const INPUT    = 1 << 3;
        const STATUS   = 1 << 4;
        const APPROVAL = 1 << 5;
        /// All widgets — used on resize and full refresh.
        const ALL = Self::SIDEBAR.bits()
            | Self::CHAT.bits()
            | Self::DIFF.bits()
            | Self::INPUT.bits()
            | Self::STATUS.bits()
            | Self::APPROVAL.bits();
    }
}

impl DirtyFlags {
    /// True if at least one flag is set.
    pub fn any(self) -> bool {
        self.bits() != 0
    }

    /// Returns all flags set (full refresh).
    pub fn all_dirty() -> Self {
        Self::ALL
    }
}
