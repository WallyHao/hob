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
    #[error("unknown provider `{0}`: pass an inline table or define it in config.toml")]
    UnknownProvider(String),
    /// The configuration file could not be read or parsed.
    #[error("cannot read `{path}`: {message}")]
    Config {
        /// Path of the configuration file.
        path: String,
        /// What went wrong.
        message: String,
    },
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
    /// The answer was valid JSON but not the shape the dialect promised.
    #[error("`{provider}` answered with an unexpected shape: {message}")]
    Shape {
        /// Provider that answered.
        provider: String,
        /// What was wrong with the answer.
        message: String,
    },
    /// The request could not be encoded for the provider's dialect.
    #[error("cannot build a `{provider}` request: {message}")]
    Request {
        /// Provider the request was for.
        provider: String,
        /// What could not be encoded.
        message: String,
    },
    /// The dialect has no equivalent for an option the request carries.
    #[error("`{provider}` does not support `{option}`")]
    Unsupported {
        /// Provider whose dialect was asked.
        provider: String,
        /// Option that has no equivalent.
        option: String,
    },
    /// The answer was larger than the client is willing to hold.
    #[error("`{provider}` answered with more than {limit} bytes")]
    BodyTooLarge {
        /// Provider that answered.
        provider: String,
        /// Cap that was exceeded, in bytes.
        limit: usize,
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

    /// Whether another attempt could plausibly succeed.
    ///
    /// Transport failures, rate limits, server errors and malformed answers
    /// are worth trying again; a rejected request, a missing key or a body
    /// over the cap will fail the same way every time.
    pub fn retryable(&self) -> bool {
        match self {
            Self::Http { .. } | Self::Decode { .. } => true,
            Self::Api { status, .. } => *status == 429 || *status >= 500,
            Self::UnknownProvider(_)
            | Self::Config { .. }
            | Self::MissingKey { .. }
            | Self::Shape { .. }
            | Self::Request { .. }
            | Self::Unsupported { .. }
            | Self::BodyTooLarge { .. } => false,
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
    use super::{Error, redact};
    use crate::provider::Secret;

    #[test]
    fn only_transient_failures_are_retryable() {
        let key = Secret::new("sk-test".to_owned());
        assert!(Error::api("p", 503, "down", &key).retryable());
        assert!(Error::api("p", 429, "slow down", &key).retryable());
        assert!(!Error::api("p", 401, "rejected", &key).retryable());
        assert!(!Error::api("p", 400, "malformed", &key).retryable());
        let too_big = Error::BodyTooLarge {
            provider: "p".to_owned(),
            limit: 10,
        };
        assert!(!too_big.retryable());
    }

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
