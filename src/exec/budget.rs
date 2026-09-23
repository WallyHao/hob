// --- exec::budget ---
// What one run may spend on model calls.
//
// The CLI sets the caps; every provider call spends from the same account, so a
// flow that loops cannot quietly turn into a bill. The account is shared by
// cloning, because a conversation borrows the state it lives in while it makes
// the call.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::effect::Failure;
use crate::provider::Usage;

/// The caps of one run; zero means no cap.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Limits {
    /// Provider calls, model listings included.
    pub(crate) calls: u64,
    /// Tokens the provider reports, summed over every answer.
    pub(crate) tokens: u64,
}

/// The account one run spends from.
#[derive(Debug, Clone)]
pub(crate) struct Budget {
    limits: Limits,
    spent: Arc<Spent>,
}

impl Budget {
    /// An account for one run.
    pub(crate) fn new(limits: Limits) -> Self {
        Self {
            limits,
            spent: Arc::new(Spent::default()),
        }
    }

    /// Charge one provider call, before it goes out. A call that would cross
    /// either cap is refused rather than made.
    pub(crate) fn call(&self) -> Result<(), Failure> {
        let made = self.spent.calls.fetch_add(1, Ordering::Relaxed);
        if self.limits.calls > 0 && made >= self.limits.calls {
            return Err(self.over("call", made));
        }
        let spent = self.spent.tokens.load(Ordering::Relaxed);
        if self.limits.tokens > 0 && spent >= self.limits.tokens {
            return Err(self.over("token", spent));
        }
        Ok(())
    }

    /// Add what an answer reported. A call can overshoot its token cap, since
    /// only the answer says how many tokens it cost; the next one is refused.
    pub(crate) fn spend(&self, usage: Usage) {
        self.spent
            .tokens
            .fetch_add(usage.total_tokens, Ordering::Relaxed);
    }

    /// The refusal: what was spent against the cap, and the flag that raises it.
    fn over(&self, what: &str, spent: u64) -> Failure {
        let (limit, flag) = if what == "call" {
            (self.limits.calls, "--max-calls")
        } else {
            (self.limits.tokens, "--max-tokens")
        };
        Failure::new(format!(
            "the run hit its {what} budget ({spent} of {limit}); raise {flag} to continue"
        ))
    }
}

#[derive(Debug, Default)]
struct Spent {
    calls: AtomicU64,
    tokens: AtomicU64,
}
