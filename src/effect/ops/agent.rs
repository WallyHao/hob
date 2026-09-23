// --- effect::ops::agent ---
// Arguments of the `agent` namespace.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::provider::Protocol;

/// Where to send the request: a registered id, or an inline spec.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Provider {
    /// A provider in the built-in registry, e.g. `"deepseek"`.
    Id(String),
    /// A provider described in the flow, for a local gateway or a test.
    Inline(Spec),
}

/// An inline provider: the fields the registry holds, plus headers.
#[derive(Debug, Deserialize)]
pub(crate) struct Spec {
    /// Name used in messages; `inline` when absent.
    #[serde(default = "inline_id")]
    pub(crate) id: String,
    /// Base URL without the endpoint path.
    pub(crate) base_url: String,
    /// Environment variable holding the key.
    pub(crate) api_key_env: String,
    /// Dialect used for requests; `openai` when absent.
    #[serde(default)]
    pub(crate) protocol: Option<Protocol>,
    /// Extra headers, e.g. a gateway's routing key.
    #[serde(default)]
    pub(crate) headers: BTreeMap<String, String>,
}

fn inline_id() -> String {
    "inline".to_owned()
}

/// Options an opening call and every later turn share.
#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct Settings {
    /// JSON shape the answer must match.
    #[serde(default)]
    pub(crate) schema: Option<Value>,
    /// Sampling temperature.
    #[serde(default)]
    pub(crate) temperature: Option<f32>,
    /// Cap on generated tokens.
    #[serde(default)]
    pub(crate) max_tokens: Option<u32>,
    /// How many provider calls one answer may take.
    #[serde(default)]
    pub(crate) max_attempts: Option<u32>,
    /// Estimated prompt budget for a conversation, 0 for no cap.
    #[serde(default)]
    pub(crate) max_prompt_tokens: Option<u32>,
    /// Show the answer on stderr while it is still arriving.
    #[serde(default)]
    pub(crate) stream: Option<bool>,
    /// Reasoning effort, forwarded as `reasoning_effort`.
    #[serde(default)]
    pub(crate) effort: Option<String>,
    /// Tools the model may ask for, as raw tool objects.
    #[serde(default)]
    pub(crate) tools: Option<Vec<Value>>,
}

impl Settings {
    /// The settings for one call: this value where set, `defaults` otherwise.
    pub(crate) fn over(&self, defaults: &Self) -> Self {
        Self {
            schema: self.schema.clone().or_else(|| defaults.schema.clone()),
            temperature: self.temperature.or(defaults.temperature),
            max_tokens: self.max_tokens.or(defaults.max_tokens),
            max_attempts: self.max_attempts.or(defaults.max_attempts),
            max_prompt_tokens: self.max_prompt_tokens.or(defaults.max_prompt_tokens),
            stream: self.stream.or(defaults.stream),
            effort: self.effort.clone().or_else(|| defaults.effort.clone()),
            tools: self.tools.clone().or_else(|| defaults.tools.clone()),
        }
    }
}

/// `agent.ask`.
#[derive(Debug, Deserialize)]
pub(crate) struct Ask {
    #[serde(default)]
    pub(crate) provider: Option<Provider>,
    #[serde(default)]
    pub(crate) model: Option<String>,
    #[serde(default)]
    pub(crate) system: Option<String>,
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    #[serde(default)]
    pub(crate) messages: Option<Vec<Value>>,
    #[serde(flatten)]
    pub(crate) settings: Settings,
}

/// `agent.open`.
#[derive(Debug, Deserialize)]
pub(crate) struct Open {
    #[serde(default)]
    pub(crate) provider: Option<Provider>,
    #[serde(default)]
    pub(crate) model: Option<String>,
    #[serde(default)]
    pub(crate) system: Option<String>,
    #[serde(flatten)]
    pub(crate) settings: Settings,
}

/// `agent.send`.
#[derive(Debug, Deserialize)]
pub(crate) struct Send {
    pub(crate) session: u64,
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    #[serde(default)]
    pub(crate) messages: Option<Vec<Value>>,
    #[serde(flatten)]
    pub(crate) settings: Settings,
}

/// `agent.push`.
#[derive(Debug, Deserialize)]
pub(crate) struct Push {
    pub(crate) session: u64,
    pub(crate) message: Value,
}

/// A handle to one conversation, for the methods that need nothing else.
#[derive(Debug, Deserialize)]
pub(crate) struct Handle {
    /// Conversation handle returned by `agent.open`.
    pub(crate) session: u64,
}

/// `agent.list`.
#[derive(Debug, Deserialize)]
pub(crate) struct List {
    #[serde(default)]
    pub(crate) provider: Option<Provider>,
}
