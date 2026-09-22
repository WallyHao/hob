// --- provider::wire ---
// Translating between the engine's request shape and the dialect a provider
// speaks.
//
// `ChatRequest` and `ChatResponse` stay dialect-neutral; everything that
// differs -- paths, auth headers, body shape, content blocks -- lives here, so
// the client stays one HTTP path and a third dialect is another module.

mod anthropic;
mod openai;

use serde::Deserialize;

use super::error::Error;
use super::spec::{Protocol, ProviderSpec};
use super::types::{ChatRequest, ChatResponse};

pub(crate) use anthropic::VERSION as ANTHROPIC_VERSION;

/// The path a chat request goes to.
pub(crate) fn chat_path(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::OpenAi => "chat/completions",
        Protocol::Anthropic => "v1/messages",
    }
}

/// The path a model listing goes to.
pub(crate) fn models_path(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::OpenAi => "models",
        Protocol::Anthropic => "v1/models",
    }
}

/// Build the body of a chat request in the provider's dialect.
pub(crate) fn request(
    spec: &ProviderSpec,
    request: &ChatRequest,
) -> Result<serde_json::Value, Error> {
    match spec.protocol {
        Protocol::OpenAi => openai::request(&spec.id, request),
        Protocol::Anthropic => anthropic::request(&spec.id, request),
    }
}

/// Parse a chat response in the provider's dialect.
pub(crate) fn response(spec: &ProviderSpec, body: &str) -> Result<ChatResponse, Error> {
    match spec.protocol {
        Protocol::OpenAi => openai::response(&spec.id, body),
        Protocol::Anthropic => anthropic::response(&spec.id, body),
    }
}

/// Parse a model listing; both dialects answer `{"data": [{"id": ...}]}`.
pub(crate) fn models(spec: &ProviderSpec, body: &str) -> Result<Vec<String>, Error> {
    let listing: Listing = serde_json::from_str(body).map_err(|source| Error::Decode {
        provider: spec.id.clone(),
        source,
    })?;
    Ok(listing.data.into_iter().map(|model| model.id).collect())
}

#[derive(Debug, Deserialize)]
struct Listing {
    data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
struct Model {
    id: String,
}
