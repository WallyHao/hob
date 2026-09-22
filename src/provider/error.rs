// --- provider::error ---
// Failures between a request and an answer. Every variant that carries a
// provider body goes through `redact` first, so an error can be pasted into an
// issue without pasting the key with it.

use thiserror::Error;

use super::secret::Secret;

/// Something went wrong reaching or decoding a provider.
#[derive(Debug, Error)]
pub enum Error {
    /// No provider is registered under that id.
    #[error("unknown provider `{0}`")]
    UnknownProvider(String),
    /// The environment variable the provider names is unset or empty.
    #[error("no API key for `{provider}`: set {env} in the environment")]
    MissingKey {
        /// Provider that needed the key.
        provider: String,
        /// Variable that was looked up.
        env: String,
    },
    /// The provider answered with a non-success status.
    #[error("`{provider}` returned {status}: {body}")]
    Api {
        /// Provider that answered.
        provider: String,
        /// HTTP status code.
        status: u16,
        /// Response body, scrubbed and truncated.
        body: String,
    },
    /// The request could not be completed: DNS, TLS, timeout, connection.
    #[error("cannot reach `{provider}`: {source}")]
    Http {
        /// Provider that was being called.
        provider: String,
        /// Underlying transport error.
        source: reqwest::Error,
    },
    /// The answer was not the JSON shape expected.
    #[error("`{provider}` answered with an unexpected shape: {source}")]
    Decode {
        /// Provider that answered.
        provider: String,
        /// Underlying parse error.
        source: serde_json::Error,
    },
}

impl Error {
    /// Build an `Api` error with the key scrubbed out of the body.
    pub(crate) fn api(provider: &str, status: u16, body: &str, secret: &Secret) -> Self {
        Self::Api {
            provider: provider.to_owned(),
            status,
            body: redact(body, secret.expose()),
        }
    }
}

/// Replace the key with `***`, then cap the body so one error stays readable.
fn redact(body: &str, secret: &str) -> String {
    let scrubbed = if secret.is_empty() {
        body.to_owned()
    } else {
        body.replace(secret, "***")
    };
    scrubbed.chars().take(400).collect()
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn the_key_never_survives_redaction() {
        let scrubbed = redact("invalid key: sk-secret (401)", "sk-secret");
        assert!(!scrubbed.contains("sk-secret"), "{scrubbed}");
        assert!(scrubbed.contains("***"), "{scrubbed}");
    }

    #[test]
    fn an_empty_key_does_not_match_everywhere() {
        assert_eq!(redact("plain", ""), "plain");
    }
}
