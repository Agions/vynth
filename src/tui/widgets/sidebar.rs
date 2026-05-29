//! Sidebar widget — file tree / session list / skills

pub struct Sidebar {
    pub active_tab: SidebarTab,
    pub file_tree: Vec<FileEntry>,
    pub scroll_offset: usize,
}

pub enum SidebarTab {
    Files,
    Sessions,
    Skills,
}

pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active_tab: SidebarTab::Files,
            file_tree: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn switch_tab(&mut self, tab: SidebarTab) {
        self.active_tab = tab;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}
