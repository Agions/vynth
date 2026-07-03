//! Input mode enum.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    Normal,
    #[default]
    Insert,
    Command,
    Search,
}
