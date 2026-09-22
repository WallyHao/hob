// --- provider::usage ---
// Token accounting, summed across the calls of one conversation.

use serde::{Deserialize, Serialize};

/// Tokens one call cost.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Tokens the model generated.
    #[serde(default)]
    pub completion_tokens: u64,
    /// Their sum, as the provider reports it.
    #[serde(default)]
    pub total_tokens: u64,
}

impl Usage {
    /// Add another call's counts to this one.
    pub fn add(&mut self, other: &Self) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }
}
