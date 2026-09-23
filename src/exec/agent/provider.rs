// --- exec::agent::provider ---
// Reaching the provider: resolving the spec and running the async client to
// completion on one runtime shared by the process.

use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use tokio::runtime::Runtime;

use crate::effect::Failure;
use crate::effect::ops::agent::Provider;
use crate::exec::budget::Budget;
use crate::provider::{ChatRequest, ChatResponse, Client, Error, Protocol, ProviderSpec};

/// Resolve the provider option: a registry id, the configured default, a config
/// entry or an inline spec.
pub(crate) fn spec(provider: Option<&Provider>) -> Result<ProviderSpec, Failure> {
    match provider {
        None => resolve(&default_provider()?),
        Some(Provider::Id(id)) => resolve(id),
        Some(Provider::Inline(spec)) => Ok(ProviderSpec {
            id: spec.id.clone(),
            base_url: spec.base_url.clone(),
            api_key_env: spec.api_key_env.clone(),
            protocol: spec.protocol.unwrap_or(Protocol::OpenAi),
            headers: spec.headers.clone(),
        }),
    }
}

fn resolve(id: &str) -> Result<ProviderSpec, Failure> {
    crate::provider::resolve(id).map_err(|error| Failure::new(error.to_string()))
}

/// The provider a call without one uses: `[defaults] provider`, else deepseek.
fn default_provider() -> Result<String, Failure> {
    let configured =
        crate::provider::defaults().map_err(|error| Failure::new(error.to_string()))?;
    Ok(configured.provider.unwrap_or_else(|| "deepseek".to_owned()))
}

/// The model: the option, then `[defaults] model`.
///
/// Ids change, so the engine does not guess one: a call with neither is an
/// error naming the option and the configuration key that would set it.
pub(crate) fn model(model: Option<&str>) -> Result<String, Failure> {
    if let Some(name) = model.filter(|name| !name.is_empty()) {
        return Ok(name.to_owned());
    }
    let configured =
        crate::provider::defaults().map_err(|error| Failure::new(error.to_string()))?;
    configured.model.ok_or_else(|| {
        Failure::new(
            "no model: pass `model`, set `[defaults] model` in config.toml, \
             or list them with `hob.agent.list`",
        )
    })
}

/// Why one call failed, and whether another attempt could help.
#[derive(Debug)]
pub(crate) struct CallError {
    /// The failure the flow would see.
    pub(crate) failure: Failure,
    /// Whether a retry could plausibly succeed.
    pub(crate) retryable: bool,
}

impl CallError {
    /// A failure that would happen again the same way.
    fn fatal(failure: Failure) -> Self {
        Self {
            failure,
            retryable: false,
        }
    }
}

impl From<Error> for CallError {
    fn from(error: Error) -> Self {
        Self {
            failure: Failure::new(error.to_string()),
            retryable: error.retryable(),
        }
    }
}

/// One provider call on the shared runtime; a streaming request shows its
/// deltas through `sink`.
pub(crate) fn call(
    client: &Client,
    request: &ChatRequest,
    sink: &mut dyn FnMut(&str),
) -> Result<ChatResponse, CallError> {
    let runtime = runtime().map_err(CallError::fatal)?;
    let result = if request.stream {
        runtime.block_on(client.chat_stream(request, sink))
    } else {
        runtime.block_on(client.chat(request))
    };
    result.map_err(CallError::from)
}

/// The models a provider lists.
pub(crate) fn models(client: &Client, budget: &Budget) -> Result<Vec<String>, Failure> {
    budget.call()?;
    let runtime = runtime()?;
    runtime
        .block_on(client.models())
        .map_err(|error| Failure::new(error.to_string()))
}

/// The runtime every provider call runs on.
///
/// One runtime for the process keeps the clients' pooled connections alive
/// between calls, so a conversation does not pay for a handshake per turn; the
/// mutex is because a current-thread runtime may only be driven by one thread
/// at a time.
static RUNTIME: OnceLock<Result<Mutex<Runtime>, String>> = OnceLock::new();

fn runtime() -> Result<MutexGuard<'static, Runtime>, Failure> {
    match RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map(Mutex::new)
            .map_err(|error| error.to_string())
    }) {
        Ok(runtime) => Ok(runtime.lock().unwrap_or_else(PoisonError::into_inner)),
        Err(message) => Err(Failure::new(format!(
            "cannot start the async runtime: {message}"
        ))),
    }
}
