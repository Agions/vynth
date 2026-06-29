//! Diff preview state types.

/// Diff preview state
#[derive(Debug, Clone, Default)]
pub struct DiffState {
    pub content: String,
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
