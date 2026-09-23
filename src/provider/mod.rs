// --- provider ---
// Model backends: what hob knows about each provider, where the key comes
// from, and the one client that talks to them.
//
// Providers that speak the OpenAI chat-completions shape are data, not code:
// adding one is an entry in `registry`. A provider with its own dialect
// becomes a new `Protocol` variant with a module beside `client`, so the seam
// for the second dialect is drawn before it is needed.
//
// Keys are read from the environment at request time and never stored on
// disk; `Secret` keeps them out of debug output and error messages.

mod body;
mod cache;
mod client;
mod config;
mod error;
mod registry;
mod secret;
mod spec;
mod stream;
mod types;
mod usage;
mod wire;

pub(crate) use cache::Cache;
pub use client::Client;
pub use error::Error;
pub use registry::{builtins, find};
pub use secret::Secret;
pub use spec::{Protocol, ProviderSpec};
pub use types::{ChatRequest, ChatResponse, Choice, FunctionCall, Message, Role, ToolCall};
pub use usage::Usage;

/// Result of a provider operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Look up a provider by id: the registry first, then the configuration file.
pub fn resolve(id: &str) -> Result<ProviderSpec> {
    if let Some(spec) = find(id) {
        return Ok(spec);
    }
    config::find(id)?.ok_or_else(|| Error::UnknownProvider(id.to_owned()))
}

/// What a call uses when the flow names neither provider nor model.
pub(crate) fn defaults() -> Result<config::Defaults> {
    Ok(config::read()?.defaults)
}

/// Every provider hob knows: the registry, then the configuration file.
pub fn known() -> Result<Vec<ProviderSpec>> {
    let mut all = builtins();
    for spec in config::all()? {
        if !all.iter().any(|known| known.id == spec.id) {
            all.push(spec);
        }
    }
    Ok(all)
}
