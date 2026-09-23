// --- exec::agent::budget ---
// Keeping a conversation inside a token budget.
//
// The engine has no tokenizer and no model table, so sizes are estimated: four
// ASCII bytes or one wide character per token, plus a little framing. A long
// conversation drops its oldest exchanges instead of failing at the provider,
// and the newest exchange is never dropped, so one oversized prompt still
// travels and gets the provider's own error.

use crate::provider::{Message, Role};

/// The prompt budget when the flow names none.
pub(crate) const DEFAULT: u32 = 32_000;

/// Tokens reserved for the marker that says history was dropped.
const MARKER: u32 = 32;

/// What a message costs, framing included.
pub(crate) fn tokens(message: &Message) -> u32 {
    let mut total = 4 + text(&message.content);
    for call in message.tool_calls.iter().flatten() {
        total += 4 + text(&call.function.name) + text(&call.function.arguments);
    }
    total
}

/// What a conversation costs.
pub(crate) fn estimate(messages: &[Message]) -> u32 {
    messages.iter().map(tokens).sum()
}

/// How many leading messages to drop so history plus `fixed` fits `budget`.
///
/// Cutting stops at a user message, so an exchange stays whole and a tool
/// result never travels without the call it answers. `fixed` is what a send
/// always carries: the system prompt and the new messages. A budget of zero
/// turns the cap off.
pub(crate) fn dropped(history: &[Message], fixed: u32, budget: u32) -> usize {
    if budget == 0 {
        return 0;
    }
    let fixed = fixed.saturating_add(MARKER);
    let mut total = estimate(history);
    let mut gone = 0;
    while fixed.saturating_add(total) > budget {
        let cut = history[gone..]
            .iter()
            .skip(1)
            .position(|message| message.role == Role::User)
            .map(|skip| skip + 1);
        let Some(cut) = cut else { break };
        total = total.saturating_sub(estimate(&history[gone..gone + cut]));
        gone += cut;
    }
    gone
}

/// What the model is told when history had to go.
pub(crate) fn marker(count: usize, budget: u32) -> String {
    format!("[hob dropped {count} earlier message(s) to keep the prompt under {budget} tokens]")
}

/// A rough token count for text: ASCII is about four bytes per token, anything
/// wider is about one character per token.
fn text(text: &str) -> u32 {
    let mut ascii = 0u32;
    let mut wide = 0u32;
    for character in text.chars() {
        if character.is_ascii() {
            ascii += 1;
        } else {
            wide += 1;
        }
    }
    ascii / 4 + wide
}
