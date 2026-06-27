//! Slash command menu filtering helpers.

use crate::slash::{CmdDef, COMMANDS};

pub const MAX_SLASH_MENU_ITEMS: usize = 6;

pub fn menu_matches(input: &str) -> Vec<&'static CmdDef> {
    if input.contains(' ') {
        return Vec::new();
    }

    let query = input.trim_start_matches('/').to_ascii_lowercase();
    if query.is_empty() && !input.starts_with('/') {
        return Vec::new();
    }

    COMMANDS
        .iter()
        .filter(|cmd| {
            let name = cmd.name.trim_start_matches('/');
            query.is_empty()
                || name.starts_with(&query)
                || cmd.aliases.iter().any(|alias| alias.contains(&query))
        })
        .take(MAX_SLASH_MENU_ITEMS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_menu_empty_slash_lists_commands() {
        let matches = menu_matches("/");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "/help");
    }

    #[test]
    fn slash_menu_filters_by_prefix() {
        let names: Vec<&str> = menu_matches("/mo")
            .into_iter()
            .map(|cmd| cmd.name)
            .collect();
        assert!(names.contains(&"/model"));
        assert!(names.contains(&"/mode"));
    }

    #[test]
    fn slash_menu_hides_after_args() {
        assert!(menu_matches("/model list").is_empty());
    }
}
