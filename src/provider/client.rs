// --- provider::client ---
// The one HTTP client, bound to a provider and a key at construction. It owns
// the two calls the engine needs: one completion and one model listing. The
// dialect -- paths, auth headers, body and response shapes -- lives in
// `wire`, so this module is only HTTP.

use std::time::Duration;

use super::error::Error;
use super::secret::{self, Secret};
use super::spec::{Protocol, ProviderSpec};
use super::types::{ChatRequest, ChatResponse};
use super::wire;

/// Long enough for a slow model, short enough that a script cannot hang all
/// day on a dead connection.
const TIMEOUT: Duration = Duration::from_secs(300);

/// A client for one provider. Holds the key in memory only.
#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    spec: ProviderSpec,
    key: Secret,
}

impl Client {
    /// Build a client from an already resolved key.
    pub fn new(spec: ProviderSpec, key: Secret) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(|source| Error::Http {
                provider: spec.id.clone(),
                source,
            })?;
        Ok(Self { http, spec, key })
    }

    /// Build a client, reading the key from the variable the provider names.
    pub fn from_env(spec: ProviderSpec) -> Result<Self, Error> {
        let key = secret::resolve(&spec, |name| std::env::var(name).ok())?;
        Self::new(spec, key)
    }

    /// The provider this client is bound to.
    pub fn spec(&self) -> &ProviderSpec {
        &self.spec
    }

    /// One chat completion.
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, Error> {
        let path = wire::chat_path(self.spec.protocol);
        let body = wire::request(&self.spec, request)?;
        let response = self
            .headers(self.authorized(self.http.post(self.spec.endpoint(path))))
            .json(&body)
            .send()
            .await
            .map_err(|source| self.transport(source))?;
        let body = self.body(response).await?;
        wire::response(&self.spec, &body)
    }

    /// The models the provider lists.
    pub async fn models(&self) -> Result<Vec<String>, Error> {
        let path = wire::models_path(self.spec.protocol);
        let response = self
            .headers(self.authorized(self.http.get(self.spec.endpoint(path))))
            .send()
            .await
            .map_err(|source| self.transport(source))?;
        let body = self.body(response).await?;
        wire::models(&self.spec, &body)
    }

    /// Authenticate the request the way the dialect expects.
    fn authorized(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.spec.protocol {
            Protocol::OpenAi => request.bearer_auth(self.key.expose()),
            Protocol::Anthropic => request
                .header("x-api-key", self.key.expose())
                .header("anthropic-version", wire::ANTHROPIC_VERSION),
        }
    }

    /// Add the extra headers a gateway needs for routing.
    fn headers(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        for (name, value) in &self.spec.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        request
    }

    /// Read the body, or turn a non-success status into an error without the key.
    async fn body(&self, response: reqwest::Response) -> Result<String, Error> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|source| self.transport(source))?;
        if !status.is_success() {
            return Err(Error::api(&self.spec.id, status.as_u16(), &body, &self.key));
        }
        Ok(body)
    }

    fn transport(&self, source: reqwest::Error) -> Error {
        Error::Http {
            provider: self.spec.id.clone(),
            source,
        }
    }
}
