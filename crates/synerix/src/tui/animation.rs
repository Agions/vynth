//! Shared TUI animation helpers.
//!
//! Centralizes frame-driven animation primitives so widgets share a single
//! implementation instead of each redefining their own (see chat_area and
//! status_bar, which both previously held an identical `animated_dots`).

/// Cycle an ellipsis (``, `.`, `..`, `...`) by animation frame.
///
/// Used for "thinking…" / "running tool…" indicators. The empty first frame
/// keeps the indicator from looking static.
pub fn animated_dots(frame: u64) -> &'static str {
    match frame % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    }
}

#[cfg(test)]
mod tests {
    use super::animated_dots;

    #[test]
    fn animated_dots_cycles_through_four_frames() {
        assert_eq!(animated_dots(0), "");
        assert_eq!(animated_dots(1), ".");
        assert_eq!(animated_dots(2), "..");
        assert_eq!(animated_dots(3), "...");
        assert_eq!(animated_dots(4), "");
    }
}
