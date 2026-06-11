//! Sidebar panel state types.

/// Sidebar panel state
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    pub active_tab: SidebarTab,
    pub file_tree: Vec<FileEntry>,
    /// Scroll offset for file list
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SidebarTab {
    #[default]
    Files,
    Sessions,
    Skills,
}

#[derive(Debug, Clone, Default)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
}
