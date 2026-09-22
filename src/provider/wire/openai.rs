// --- provider::wire::openai ---
// The chat-completions dialect: the request is the engine's own shape and the
// answer is `choices[].message`, so there is nothing to translate.

use crate::provider::error::Error;
use crate::provider::types::{ChatRequest, ChatResponse};

/// The request body: the engine's shape, as it stands.
pub(crate) fn request(provider: &str, request: &ChatRequest) -> Result<serde_json::Value, Error> {
    serde_json::to_value(request).map_err(|error| Error::Request {
        provider: provider.to_owned(),
        message: error.to_string(),
    })
}

/// The answer, decoded as the engine's shape.
pub(crate) fn response(provider: &str, body: &str) -> Result<ChatResponse, Error> {
    serde_json::from_str(body).map_err(|source| Error::Decode {
        provider: provider.to_owned(),
        source,
    })
}
