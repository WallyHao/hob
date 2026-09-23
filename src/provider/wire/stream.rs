// --- provider::wire::stream ---
// Incremental readers for the two dialects' streaming answers.
//
// A stream is still one answer: the deltas are shown as they arrive, and the
// decoder assembles the same `ChatResponse` a non-streaming call would have
// given, so nothing above the client branches on how the answer travelled.

use super::{anthropic, openai};
use crate::provider::error::Error;
use crate::provider::spec::{Protocol, ProviderSpec};
use crate::provider::types::ChatResponse;

/// What a dialect must do to turn events into one answer.
pub(crate) trait Decode {
    /// Read one `data:` payload; returns the text to show now, if any.
    fn feed(&mut self, data: &str) -> Result<Option<String>, Error>;
    /// The answer the events added up to.
    fn finish(&mut self, provider: &str) -> Result<ChatResponse, Error>;
    fn complete(&self) -> bool;
}

/// The reader for a provider's dialect.
pub(crate) fn decoder(spec: &ProviderSpec) -> Box<dyn Decode> {
    match spec.protocol {
        Protocol::OpenAi => Box::new(openai::Stream::new(&spec.id)),
        Protocol::Anthropic => Box::new(anthropic::Stream::new(&spec.id)),
    }
}
