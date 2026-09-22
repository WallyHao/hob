// --- provider::secret ---
// API keys: read from the environment, never written down, never printed.
//
// hob deliberately has no credential file. A secret at rest is a secret that
// ends up in a dotfiles repository, a backup or a screen share, and storing it
// well (permissions, encryption, rotation) is a responsibility a personal CLI
// should not take on. A key that only lives in the process environment
// composes with direnv, `pass`, `op run` or a systemd unit instead.

use std::fmt;

use super::error::Error;
use super::spec::ProviderSpec;

/// An API key that cannot be printed by accident.
///
/// `Debug` is redacted, there is no `Display` and no serialisation, so the key
/// can only leave through `expose`, which marks the request boundary as the
/// one place worth auditing.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    /// Wrap a key that came from somewhere already trusted.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// The key itself.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(\"***\")")
    }
}

/// Resolve the key a provider names through `lookup`.
///
/// The lookup is injected instead of reading `std::env` here so tests stay
/// deterministic and free of process-global state; production passes the
/// environment.
pub(super) fn resolve(
    spec: &ProviderSpec,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<Secret, Error> {
    lookup(&spec.api_key_env)
        .map(Secret::new)
        .ok_or_else(|| Error::MissingKey {
            provider: spec.id.clone(),
            env: spec.api_key_env.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::{Secret, resolve};
    use crate::provider::registry::find;

    #[test]
    fn a_missing_variable_names_itself() {
        let spec = find("deepseek").expect("registered");
        let error = resolve(&spec, |_| None).expect_err("nothing in the environment");
        assert!(error.to_string().contains("DEEPSEEK_API_KEY"), "{error}");
    }

    #[test]
    fn a_present_variable_becomes_the_key() {
        let spec = find("deepseek").expect("registered");
        let secret = resolve(&spec, |name| {
            (name == "DEEPSEEK_API_KEY").then(|| "sk-from-env".to_owned())
        })
        .expect("the lookup answers");
        assert_eq!(secret.expose(), "sk-from-env");
    }

    #[test]
    fn debug_output_is_redacted() {
        let secret = Secret::new("sk-do-not-print".to_owned());
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains("sk-do-not-print"), "{rendered}");
    }
}
