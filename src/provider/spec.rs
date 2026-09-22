// --- provider::spec ---
// What the registry knows about a provider, and what it deliberately does not:
// no key, and no model. The key comes from the environment, and the model is
// the caller's choice because it changes per request.

use std::collections::BTreeMap;

use serde::Deserialize;

/// The wire dialect a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// `OpenAI` chat completions, spoken by `DeepSeek` and by most gateways
    /// that copy it, so one client covers all of them.
    OpenAi,
}

/// Everything needed to reach a provider except the key.
#[derive(Debug, Clone)]
pub struct ProviderSpec {
    /// Stable id used in configuration and error messages.
    pub id: String,
    /// Base URL without the endpoint path, e.g. `https://api.deepseek.com`.
    pub base_url: String,
    /// Environment variable that holds the key; never the key itself.
    pub api_key_env: String,
    /// Dialect used for requests.
    pub protocol: Protocol,
    /// Extra headers a gateway needs for routing.
    pub headers: BTreeMap<String, String>,
}

impl ProviderSpec {
    /// Build a spec with no extra headers.
    pub fn new(id: &str, base_url: &str, api_key_env: &str, protocol: Protocol) -> Self {
        Self {
            id: id.to_owned(),
            base_url: base_url.to_owned(),
            api_key_env: api_key_env.to_owned(),
            protocol,
            headers: BTreeMap::new(),
        }
    }

    /// The URL for an endpoint path under the base URL.
    pub fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}
