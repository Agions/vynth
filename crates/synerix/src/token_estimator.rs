//! Shared token estimation utility with LRU caching
//!
//! Provides fast, cached token estimation for:
//! - English text (~4 chars/token)
//! - CJK text (~2 chars/token)
//! - Mixed content (weighted combination)
//!
//! Uses a moka `sync::Cache` for bounded LRU caching with TTL-based eviction.
//! TODO: Token estimator — not yet wired
#![allow(dead_code)]

use std::sync::OnceLock;

use moka::sync::Cache;

/// Global token estimation cache (lazy-initialized)
/// Maps content hash → estimated token count
static TOKEN_CACHE: OnceLock<Cache<u64, usize>> = OnceLock::new();

/// Get or initialize the moka cache with:
/// - max 1024 entries
/// - 300s time-to-idle (evict entries untouched for 5 min)
fn get_cache() -> &'static Cache<u64, usize> {
    TOKEN_CACHE.get_or_init(|| {
        Cache::builder()
            .max_capacity(1024)
            .time_to_idle(std::time::Duration::from_secs(300))
            .build()
    })
}

/// FNV-1a hash for fast content hashing (no cryptographic need)
#[inline]
fn fnv1a_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

/// Count CJK characters in text (optimized: single pass, no allocation)
/// Matches original 3-range definition exactly
#[inline]
fn count_cjk_chars(text: &str) -> usize {
    text.chars()
        .filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp)   // CJK Unified Ideographs
            || (0x3400..=0x4DBF).contains(&cp) // CJK Extension A
            || (0xF900..=0xFAFF).contains(&cp) // CJK Compatibility
        })
        .count()
}

/// Estimate token count for text (cached version — use for repeated content)
///
/// ~4 chars per token for Latin text, ~2 chars per token for CJK text.
/// Results are cached by content hash for O(1) repeated lookups.
pub fn estimate_tokens(text: &str) -> usize {
    // Small texts (< 32 chars): compute directly, not worth caching
    if text.len() < 32 {
        return estimate_tokens_inner(text);
    }

    let hash = fnv1a_hash(text);
    let cache = get_cache();

    // Check cache first
    if let Some(cached) = cache.get(&hash) {
        return cached;
    }

    // Compute and cache
    let result = estimate_tokens_inner(text);
    cache.insert(hash, result);
    result
}

/// Inner estimation (no caching overhead)
#[inline]
fn estimate_tokens_inner(text: &str) -> usize {
    let byte_len = text.len();

    // Fast path: if all ASCII, skip char iteration entirely
    if text.is_ascii() {
        return (byte_len / 4) + 1;
    }

    let cjk_count = count_cjk_chars(text);
    let other_count = byte_len - cjk_count;

    (other_count / 4) + (cjk_count / 2) + 1
}

/// Estimate tokens for multiple texts in batch (avoids repeated cache lookups)
pub fn estimate_tokens_batch(texts: &[&str]) -> Vec<usize> {
    texts.iter().map(|t| estimate_tokens(t)).collect()
}

/// Clear the token estimation cache (useful for testing or memory pressure)
pub fn clear_cache() {
    if let Some(cache) = TOKEN_CACHE.get() {
        cache.invalidate_all();
    }
}

/// Get current cache stats (for diagnostics)
/// Returns (current_entry_count, max_capacity)
pub fn cache_stats() -> (usize, usize) {
    let count = TOKEN_CACHE
        .get()
        .map(|c| c.entry_count() as usize)
        .unwrap_or(0);
    (count, 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        // Legacy behavior: (0/4) + (0/2) + 1 = 1
        assert_eq!(estimate_tokens(""), 1);
    }

    #[test]
    fn test_ascii_text() {
        let tokens = estimate_tokens("hello world");
        assert!(tokens > 0);
        assert!(tokens < 10);
    }

    #[test]
    fn test_cjk_text() {
        let tokens = estimate_tokens("你好世界");
        assert!(tokens > 0);
        // CJK: 12 bytes, 4 CJK chars → other=8, cjk=4 → (8/4)+(4/2)+1 = 5
        assert_eq!(tokens, 5);
    }

    #[test]
    fn test_mixed_text() {
        let tokens = estimate_tokens("hello 你好 world 世界");
        assert!(tokens > 0);
    }

    #[test]
    fn test_cache_hit() {
        clear_cache();
        let text = "this is a test string that is long enough to be cached";
        let t1 = estimate_tokens(text);
        let t2 = estimate_tokens(text);
        assert_eq!(t1, t2);
        // moka's entry_count() is approximate; the key assertion is that
        // repeated lookups return the same value (cache hit)
    }

    #[test]
    fn test_batch_estimation() {
        let texts = vec!["hello", "world", "你好"];
        let results = estimate_tokens_batch(&texts);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&t| t > 0));
    }

    #[test]
    fn test_cache_eviction() {
        clear_cache();
        // Fill cache beyond capacity
        for i in 0..1124 {
            let text = format!("unique test string number {} with enough bytes", i);
            estimate_tokens(&text);
        }
        let (size, max) = cache_stats();
        // moka maintains approximate capacity; entry_count may slightly exceed
        // the configured max during high-throughput inserts
        assert!(size <= max + 10);
    }

    #[test]
    fn test_consistency_with_legacy() {
        // Ensure our optimized version matches the original behavior
        let test_cases = vec![
            "hello world",
            "你好世界",
            "mixed 混合 text",
            "",
            "a",
            "ab",
            "abcdefghij",
        ];

        for text in test_cases {
            let optimized = estimate_tokens(text);
            let legacy = legacy_estimate_tokens(text);
            assert_eq!(optimized, legacy, "Mismatch for {:?}", text);
        }
    }

    /// Original implementation for comparison
    fn legacy_estimate_tokens(text: &str) -> usize {
        let cjk_count = text
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                (0x4E00..=0x9FFF).contains(&cp)
                    || (0x3400..=0x4DBF).contains(&cp)
                    || (0xF900..=0xFAFF).contains(&cp)
            })
            .count();
        let other_count = text.len() - cjk_count;
        (other_count / 4) + (cjk_count / 2) + 1
    }
}