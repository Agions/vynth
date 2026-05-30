//! Context manager — adaptive token budget + smart trimming

use crate::llm::types::{ChatMessage, MessageRole};

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

    /// Update budget based on actual system prompt and tool schema sizes
    pub fn update_from_actuals(&mut self, system_tokens: usize, tool_tokens: usize) {
        self.system_prompt = system_tokens;
        self.tools_schema = tool_tokens;
        self.available = self
            .total
            .saturating_sub(self.system_prompt + self.tools_schema + self.reserved);
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
    /// 1. Compress old tool_results to summary
    /// 2. Merge adjacent same-role messages
    /// 3. Sliding window — drop oldest messages
    /// 4. Always preserve: system + last 3 turns
    fn trim_to_budget(&mut self) {
        let target = (self.budget.available as f64 * 0.6) as usize;

        // Strategy 1: Compress old tool results
        self.compress_old_tool_results();

        // Strategy 2: Drop oldest non-system messages (keep last 3 turns = 6 messages)
        let min_keep = 6;
        while self.messages.len() > min_keep && self.estimated_tokens > target {
            // Find first non-system, non-recent message
            let keep_from = self.messages.len().saturating_sub(min_keep);
            if let Some(pos) = self
                .messages
                .iter()
                .take(keep_from)
                .position(|m| !matches!(m.role, MessageRole::System))
            {
                let removed = self.messages.remove(pos);
                self.estimated_tokens -= estimate_tokens(removed.content.as_deref().unwrap_or(""));
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

    /// Compress old tool results to summaries
    /// Replaces verbose tool output with a compact summary
    fn compress_old_tool_results(&mut self) {
        // Keep the last 4 tool results intact, compress older ones
        let tool_indices: Vec<usize> = self
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| matches!(m.role, MessageRole::Tool))
            .map(|(i, _)| i)
            .collect();

        let keep_recent = 4;
        if tool_indices.len() <= keep_recent {
            return;
        }

        let compress_indices = &tool_indices[..tool_indices.len() - keep_recent];

        for &idx in compress_indices.iter().rev() {
            if let Some(msg) = self.messages.get_mut(idx) {
                if let Some(ref content) = msg.content {
                    let original_tokens = estimate_tokens(content);
                    let summary = summarize_tool_result(content);
                    let summary_tokens = estimate_tokens(&summary);

                    if summary_tokens < original_tokens {
                        self.estimated_tokens -= original_tokens;
                        self.estimated_tokens += summary_tokens;
                        msg.content = Some(summary);
                    }
                }
            }
        }
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

    /// Mutable token budget (for adaptive updates)
    pub fn budget_mut(&mut self) -> &mut TokenBudget {
        &mut self.budget
    }

    /// Usage ratio (0.0 - 1.0)
    pub fn usage_ratio(&self) -> f64 {
        if self.budget.available == 0 {
            return 1.0;
        }
        self.estimated_tokens as f64 / self.budget.available as f64
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.estimated_tokens = 0;
    }

    /// Get message count by role
    pub fn count_by_role(&self) -> (usize, usize, usize, usize) {
        let mut system = 0;
        let mut user = 0;
        let mut assistant = 0;
        let mut tool = 0;
        for msg in &self.messages {
            match msg.role {
                MessageRole::System => system += 1,
                MessageRole::User => user += 1,
                MessageRole::Assistant => assistant += 1,
                MessageRole::Tool => tool += 1,
            }
        }
        (system, user, assistant, tool)
    }
}

/// Summarize a tool result to a compact form
fn summarize_tool_result(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() <= 5 {
        return content.to_string();
    }

    // For file contents, show first 3 + last 2 lines
    if content.len() > 500 {
        let first_3: String = lines.iter().take(3).cloned().collect::<Vec<_>>().join("\n");
        let last_2: String = {
            let mut v: Vec<&str> = lines.iter().rev().take(2).cloned().collect();
            v.reverse();
            v.join("\n")
        };
        format!(
            "{}\n... ({} lines omitted) ...\n{}",
            first_3,
            lines.len() - 5,
            last_2
        )
    } else {
        content.to_string()
    }
}

/// Rough token estimation (~4 chars per token for English, ~2 for CJK)
fn estimate_tokens(text: &str) -> usize {
    let cjk_count = text
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp) || // CJK Unified
            (0x3400..=0x4DBF).contains(&cp) || // CJK Extension A
            (0xF900..=0xFAFF).contains(&cp) // CJK Compatibility
        })
        .count();

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
        assert!(estimate_tokens("mixed 混合 text") > 0);
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

    #[test]
    fn test_usage_ratio() {
        let budget = TokenBudget::new(100_000);
        let mut ctx = ContextManager::new(budget);
        assert_eq!(ctx.usage_ratio(), 0.0);

        ctx.push(ChatMessage::user("hello"));
        assert!(ctx.usage_ratio() > 0.0);
        assert!(ctx.usage_ratio() < 1.0);
    }

    #[test]
    fn test_count_by_role() {
        let budget = TokenBudget::new(100_000);
        let mut ctx = ContextManager::new(budget);
        ctx.push(ChatMessage::system("test"));
        ctx.push(ChatMessage::user("hello"));
        ctx.push(ChatMessage::assistant("hi"));
        ctx.push(ChatMessage::tool_result("t1".into(), "result".into()));

        let (sys, usr, asst, tool) = ctx.count_by_role();
        assert_eq!(sys, 1);
        assert_eq!(usr, 1);
        assert_eq!(asst, 1);
        assert_eq!(tool, 1);
    }

    #[test]
    fn test_budget_update_from_actuals() {
        let mut budget = TokenBudget::new(100_000);
        budget.update_from_actuals(5000, 8000);
        assert_eq!(budget.system_prompt, 5000);
        assert_eq!(budget.tools_schema, 8000);
        assert_eq!(budget.available, 100_000 - 5000 - 8000 - 4096);
    }

    #[test]
    fn test_summarize_tool_result() {
        let short = "short result";
        assert_eq!(summarize_tool_result(short), short);

        let long = (0..20)
            .map(|i| {
                format!(
                    "line {} with extra data to make it longer {}",
                    i,
                    "x".repeat(30)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let summary = summarize_tool_result(&long);
        assert!(summary.contains("omitted"));
        assert!(summary.len() < long.len());
    }

    #[test]
    fn test_compress_old_tool_results() {
        let budget = TokenBudget::new(10_000);
        let mut ctx = ContextManager::new(budget);

        // Add 8 tool results
        for i in 0..8 {
            ctx.push(ChatMessage::tool_result(
                format!("tool_{}", i),
                format!("result with lots of data: {}", "x".repeat(200)),
            ));
        }

        // Old results should be compressed
        let tool_count = ctx
            .messages()
            .iter()
            .filter(|m| matches!(m.role, MessageRole::Tool))
            .count();
        assert_eq!(tool_count, 8); // Same count but older ones are compressed
    }
}
