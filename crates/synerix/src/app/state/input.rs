//! Input mode enum.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputMode {
    Normal,
    #[default]
    Insert,
    Command,
    Search,
}
