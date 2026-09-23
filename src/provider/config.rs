// --- provider::config ---
// Providers and defaults from the user's configuration file.
//
// A provider that is not in the registry may be defined in `config.toml` under
// `[providers.<id>]`, so a local gateway or an internal endpoint does not need
// an inline table in every flow. The registry wins for an id it knows: that is
// what a name like `deepseek` means.
//
// `[defaults]` holds what a call uses when the flow names neither: the provider
// to resolve and the model to send. Keys stay in the environment: an entry
// names the variable to read, never the key itself.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::error::Error;
use super::spec::{Protocol, ProviderSpec};
use crate::paths;

/// The shape of `config.toml`.
#[derive(Debug, Default, Deserialize)]
pub(super) struct File {
    #[serde(default)]
    providers: BTreeMap<String, Entry>,
    #[serde(default)]
    pub(super) defaults: Defaults,
}

/// The `[defaults]` table.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Defaults {
    /// Provider id used when a call names none.
    #[serde(default)]
    pub(crate) provider: Option<String>,
    /// Model id used when a call names none.
    #[serde(default)]
    pub(crate) model: Option<String>,
}

/// One `[providers.<id>]` table.
#[derive(Debug, Deserialize)]
struct Entry {
    /// Base URL without the endpoint path.
    base_url: String,
    /// Environment variable that holds the key; never the key itself.
    api_key_env: String,
    /// Dialect used for requests; `openai` when absent.
    #[serde(default)]
    protocol: Option<Protocol>,
    /// Extra headers a gateway needs for routing.
    #[serde(default)]
    headers: BTreeMap<String, String>,
}

/// Look up a provider defined in the user's configuration file.
pub(crate) fn find(id: &str) -> Result<Option<ProviderSpec>, Error> {
    Ok(all()?.into_iter().find(|spec| spec.id == id))
}

/// Parse the configuration file; a missing file is an empty one.
pub(super) fn read() -> Result<File, Error> {
    let Some(path) = paths::config_dir().map(|dir| dir.join("config.toml")) else {
        return Ok(File::default());
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(File::default()),
        Err(error) => {
            return Err(Error::Config {
                path: path.display().to_string(),
                message: error.to_string(),
            });
        }
    };
    toml::from_str(&text).map_err(|error| Error::Config {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

/// Every provider the configuration file defines.
pub(crate) fn all() -> Result<Vec<ProviderSpec>, Error> {
    Ok(read()?
        .providers
        .into_iter()
        .map(|(id, entry)| ProviderSpec {
            id,
            base_url: entry.base_url,
            api_key_env: entry.api_key_env,
            protocol: entry.protocol.unwrap_or(Protocol::OpenAi),
            headers: entry.headers,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::File;

    #[test]
    fn an_entry_keeps_its_fields() {
        let file: File = toml::from_str(
            r#"
            [providers.local]
            base_url = "http://127.0.0.1:8080"
            api_key_env = "LOCAL_KEY"
            headers = { X-Route = "team" }
            "#,
        )
        .expect("the file parses");
        let entry = file.providers.get("local").expect("the entry");
        assert_eq!(entry.base_url, "http://127.0.0.1:8080");
        assert_eq!(entry.api_key_env, "LOCAL_KEY");
        assert_eq!(entry.headers["X-Route"], "team");
    }

    #[test]
    fn a_file_without_providers_is_empty_not_an_error() {
        let file: File = toml::from_str("").expect("an empty file parses");
        assert!(file.providers.is_empty());
    }
}
