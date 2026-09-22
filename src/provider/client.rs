// --- provider::client ---
// The one HTTP client, bound to a provider and a key at construction. It owns
// the two calls the engine needs: one completion and one model listing.

use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::error::Error;
use super::secret::{self, Secret};
use super::spec::{Protocol, ProviderSpec};
use super::types::{ChatRequest, ChatResponse};

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
        let path = match self.spec.protocol {
            Protocol::OpenAi => "chat/completions",
        };
        let response = self
            .headers(
                self.http
                    .post(self.spec.endpoint(path))
                    .bearer_auth(self.key.expose()),
            )
            .json(request)
            .send()
            .await
            .map_err(|source| self.transport(source))?;
        self.decode(response).await
    }

    /// The models the provider lists.
    pub async fn models(&self) -> Result<Vec<String>, Error> {
        let path = match self.spec.protocol {
            Protocol::OpenAi => "models",
        };
        let response = self
            .headers(
                self.http
                    .get(self.spec.endpoint(path))
                    .bearer_auth(self.key.expose()),
            )
            .send()
            .await
            .map_err(|source| self.transport(source))?;
        let listing: ModelsResponse = self.decode(response).await?;
        Ok(listing.data.into_iter().map(|model| model.id).collect())
    }

    /// Add the extra headers a gateway needs for routing.
    fn headers(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        for (name, value) in &self.spec.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        request
    }

    /// Turn a response into `T`, or into an error that cannot contain the key.
    async fn decode<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T, Error> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|source| self.transport(source))?;
        if !status.is_success() {
            return Err(Error::api(&self.spec.id, status.as_u16(), &body, &self.key));
        }
        serde_json::from_str(&body).map_err(|source| Error::Decode {
            provider: self.spec.id.clone(),
            source,
        })
    }

    fn transport(&self, source: reqwest::Error) -> Error {
        Error::Http {
            provider: self.spec.id.clone(),
            source,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}
