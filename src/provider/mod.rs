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

mod client;
mod error;
mod registry;
mod secret;
mod spec;
mod types;

pub use client::Client;
pub use error::Error;
pub use registry::{builtins, find};
pub use secret::Secret;
pub use spec::{Protocol, ProviderSpec};
pub use types::{ChatRequest, ChatResponse, Choice, Message, Role};

/// Result of a provider operation.
pub type Result<T> = std::result::Result<T, Error>;
