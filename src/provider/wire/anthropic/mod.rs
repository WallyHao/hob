// --- provider::wire::anthropic ---
// The Messages dialect.
//
// Three things differ from chat completions: the system prompt is its own field
// rather than a message, content is a list of typed blocks, and a tool call
// carries its arguments as an object rather than a JSON string. `max_tokens` is
// required here, so a request without one gets a default; `reasoning_effort`
// has no equivalent and is refused rather than dropped.

mod build;
mod parse;

pub(crate) use build::request;
pub(crate) use parse::response;

/// The API version this module speaks.
pub(crate) const VERSION: &str = "2023-06-01";
