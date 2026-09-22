// --- exec::agent::provider ---
// Reaching the provider: resolving the spec, reading the key and running the
// async client to completion on a private runtime.

use crate::effect::Failure;
use crate::effect::ops::agent::Provider;
use crate::provider::{ChatRequest, ChatResponse, Client, Protocol, ProviderSpec};

/// Resolve the provider option: a registry id or an inline spec.
pub(crate) fn spec(provider: Option<&Provider>) -> Result<ProviderSpec, Failure> {
    match provider {
        None => {
            crate::provider::find("deepseek").ok_or_else(|| Failure::new("no default provider"))
        }
        Some(Provider::Id(id)) => crate::provider::find(id)
            .ok_or_else(|| Failure::new(format!("unknown provider `{id}`"))),
        Some(Provider::Inline(spec)) => Ok(ProviderSpec {
            id: spec.id.clone(),
            base_url: spec.base_url.clone(),
            api_key_env: spec.api_key_env.clone(),
            protocol: Protocol::OpenAi,
            headers: spec.headers.clone(),
        }),
    }
}

/// The model is required: ids change, so the engine does not guess one.
pub(crate) fn model(model: Option<&str>) -> Result<String, Failure> {
    model
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Failure::new("no model: pass `model`, or list them with `hob.agent.list`"))
}

/// One provider call.
pub(crate) fn call(spec: &ProviderSpec, request: &ChatRequest) -> Result<ChatResponse, Failure> {
    let client = client(spec)?;
    block_on(client.chat(request))
}

/// The models a provider lists.
pub(crate) fn models(spec: &ProviderSpec) -> Result<Vec<String>, Failure> {
    let client = client(spec)?;
    block_on(client.models())
}

fn client(spec: &ProviderSpec) -> Result<Client, Failure> {
    Client::from_env(spec.clone()).map_err(|error| Failure::new(error.to_string()))
}

fn block_on<T>(
    future: impl std::future::Future<Output = Result<T, crate::provider::Error>>,
) -> Result<T, Failure> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| Failure::new(format!("cannot start the async runtime: {error}")))?;
    runtime
        .block_on(future)
        .map_err(|error| Failure::new(error.to_string()))
}
