//! Token usage tracking across sessions and providers.

use std::collections::HashMap;

use crate::TokenUsage;

/// Tracks cumulative token usage per provider and per session.
pub struct TokenTracker {
    /// Total usage across all providers.
    total: TokenUsage,
    /// Usage broken down by provider name.
    by_provider: HashMap<String, TokenUsage>,
    /// Number of API calls made.
    call_count: u64,
}

impl TokenTracker {
    pub fn new() -> Self {
        Self {
            total: TokenUsage::default(),
            by_provider: HashMap::new(),
            call_count: 0,
        }
    }

    /// Record token usage from an API call.
    pub fn record(&mut self, provider: &str, usage: &TokenUsage) {
        self.total.input_tokens += usage.input_tokens;
        self.total.output_tokens += usage.output_tokens;
        self.call_count += 1;

        let entry = self.by_provider.entry(provider.to_string()).or_default();
        entry.input_tokens += usage.input_tokens;
        entry.output_tokens += usage.output_tokens;
    }

    /// Get total token usage.
    pub fn total(&self) -> &TokenUsage {
        &self.total
    }

    /// Get usage for a specific provider.
    pub fn for_provider(&self, provider: &str) -> Option<&TokenUsage> {
        self.by_provider.get(provider)
    }

    /// Get total tokens (input + output).
    pub fn total_tokens(&self) -> u64 {
        self.total
            .input_tokens
            .saturating_add(self.total.output_tokens)
    }

    /// Get number of API calls.
    pub fn call_count(&self) -> u64 {
        self.call_count
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        self.total = TokenUsage::default();
        self.by_provider.clear();
        self.call_count = 0;
    }
}

impl Default for TokenTracker {
    fn default() -> Self {
        Self::new()
    }
}
