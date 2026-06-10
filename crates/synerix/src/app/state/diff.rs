//! Diff preview state types.

/// Diff preview state
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DiffState {
    pub visible: bool,
    pub content: String,
    pub hunks: Vec<DiffHunk>,
    /// Scroll offset for diff content
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Default)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DiffLineKind {
    Add,
    Remove,
    #[default]
    Context,
}
