//! Model capability lookup shared by config, slash commands, and TUI state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub context_window: usize,
    pub max_output_tokens: usize,
}

pub const LEGACY_DEFAULT_CONTEXT_WINDOW: usize = 128_000;
pub const DEFAULT_OUTPUT_TOKENS: usize = 8192;

pub fn infer_model_capabilities(model: &str) -> Option<ModelCapabilities> {
    let normalized = model.to_ascii_lowercase();
    let name = normalized.as_str();

    let context_window = if contains_any(
        name,
        &["gpt-4.1", "gpt-4.5", "gemini-1.5-pro", "gemini-2.5"],
    ) {
        1_000_000
    } else if contains_any(
        name,
        &[
            "claude-3-7",
            "claude-4",
            "claude-opus-4",
            "claude-sonnet-4",
            "o3",
            "o4-",
            "o4_",
        ],
    ) {
        200_000
    } else if contains_any(name, &["gpt-4o", "gpt-4-turbo", "deepseek-v4", "mimo-v2.5"]) {
        128_000
    } else if contains_any(name, &["deepseek-chat", "deepseek-reasoner"]) {
        64_000
    } else if name.contains("gpt-3.5") {
        16_000
    } else {
        return None;
    };

    Some(ModelCapabilities {
        context_window,
        max_output_tokens: default_output_tokens(context_window),
    })
}

fn contains_any(input: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| input.contains(pattern))
}

pub fn default_output_tokens(context_window: usize) -> usize {
    (context_window / 16).clamp(4096, 32_768)
}

pub fn apply_model_capabilities(
    model: &str,
    context_window: &mut usize,
    max_output_tokens: &mut usize,
) -> bool {
    let Some(capabilities) = infer_model_capabilities(model) else {
        return false;
    };
    *context_window = capabilities.context_window;
    *max_output_tokens = capabilities.max_output_tokens;
    true
}

#[cfg(test)]
mod tests {
    use super::{apply_model_capabilities, infer_model_capabilities};

    #[test]
    fn infer_openai_large_context_models() {
        let caps = infer_model_capabilities("gpt-4.1").unwrap();
        assert_eq!(caps.context_window, 1_000_000);
    }

    #[test]
    fn infer_claude_sonnet_four_context() {
        let caps = infer_model_capabilities("claude-sonnet-4").unwrap();
        assert_eq!(caps.context_window, 200_000);
    }

    #[test]
    fn infer_deepseek_reasoner_context() {
        let caps = infer_model_capabilities("deepseek-reasoner").unwrap();
        assert_eq!(caps.context_window, 64_000);
    }

    #[test]
    fn apply_model_capabilities_updates_limits() {
        let mut context_window = 128_000;
        let mut max_output_tokens = 8192;
        assert!(apply_model_capabilities(
            "gpt-4.1",
            &mut context_window,
            &mut max_output_tokens
        ));
        assert_eq!(context_window, 1_000_000);
        assert_eq!(max_output_tokens, 32_768);
    }
}
