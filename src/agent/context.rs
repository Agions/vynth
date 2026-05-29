//! Context manager — token budget + dynamic trimming

use crate::llm::types::ChatMessage;

/// Token budget allocation strategy
pub struct TokenBudget {
    /// Total context window
    pub total: usize,
    /// System prompt overhead
    pub system_prompt: usize,
    /// Tool schemas overhead
    pub tools_schema: usize,
    /// Reserved for generation
    pub reserved: usize,
    /// Available = total - system - tools - reserved
    pub available: usize,
}

impl TokenBudget {
    pub fn new(total: usize) -> Self {
        let system_prompt = 2000; // Estimated
        let tools_schema = 3000; // Estimated
        let reserved = 4096;
        let available = total.saturating_sub(system_prompt + tools_schema + reserved);

        Self {
            total,
            system_prompt,
            tools_schema,
            reserved,
            available,
        }
    }
}

/// Dynamic context trimming with token budget
pub struct ContextManager {
    budget: TokenBudget,
    messages: Vec<ChatMessage>,
    estimated_tokens: usize,
}

impl ContextManager {
    pub fn new(budget: TokenBudget) -> Self {
        Self {
            budget,
            messages: Vec::new(),
            estimated_tokens: 0,
        }
    }

    /// Add a message and auto-trim if needed
    pub fn push(&mut self, msg: ChatMessage) {
        let tokens = estimate_tokens(msg.content.as_deref().unwrap_or(""));
        self.estimated_tokens += tokens;
        self.messages.push(msg);

        // Auto-trim when over 80% budget
        if self.estimated_tokens > (self.budget.available as f64 * 0.8) as usize {
            self.trim_to_budget();
        }
    }

    /// Trim strategy (priority low→high):
    /// 1. Merge adjacent same-role messages
    /// 2. Compress old tool_results to summary
    /// 3. Sliding window — drop oldest messages
    /// 4. Always preserve: system + last 3 turns
    fn trim_to_budget(&mut self) {
        let target = (self.budget.available as f64 * 0.6) as usize;

        // Strategy 1: Drop oldest non-system messages (keep last 3 turns = 6 messages)
        let min_keep = 6;
        while self.messages.len() > min_keep && self.estimated_tokens > target {
            // Find first non-system message
            if let Some(pos) = self.messages.iter().position(|m| {
                !matches!(m.role, crate::llm::types::MessageRole::System)
            }) {
                let removed = self.messages.remove(pos);
                self.estimated_tokens -=
                    estimate_tokens(removed.content.as_deref().unwrap_or(""));
            } else {
                break;
            }
        }

        tracing::debug!(
            "Context trimmed: {} messages, ~{} tokens (budget: {})",
            self.messages.len(),
            self.estimated_tokens,
            self.budget.available
        );
    }

    /// Get current messages
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get mutable messages
    pub fn messages_mut(&mut self) -> &mut Vec<ChatMessage> {
        &mut self.messages
    }

    /// Current estimated token usage
    pub fn current_tokens(&self) -> usize {
        self.estimated_tokens
    }

    /// Token budget
    pub fn budget(&self) -> &TokenBudget {
        &self.budget
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.estimated_tokens = 0;
    }
}

/// Rough token estimation (~4 chars per token for English, ~2 for CJK)
fn estimate_tokens(text: &str) -> usize {
    let cjk_count = text.chars().filter(|c| {
        let cp = *c as u32;
        (0x4E00..=0x9FFF).contains(&cp) || // CJK Unified
        (0x3400..=0x4DBF).contains(&cp) || // CJK Extension A
        (0xF900..=0xFAFF).contains(&cp) // CJK Compatibility
    }).count();

    let other_count = text.len() - cjk_count;

    (other_count / 4) + (cjk_count / 2) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        assert!(estimate_tokens("hello world") > 0);
        assert!(estimate_tokens("你好世界") > 0);
    }

    #[test]
    fn test_context_push_and_trim() {
        let budget = TokenBudget::new(1000);
        let mut ctx = ContextManager::new(budget);

        for i in 0..100 {
            ctx.push(ChatMessage::user(&format!("Message {}", i)));
        }

        // Should have been trimmed
        assert!(ctx.messages().len() < 100);
    }
}
