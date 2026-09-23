// --- provider::cache ---
// One client per provider, for the life of one flow.
//
// Building a client per request would rebuild the connection pool and re-read
// the key every time, so the turns of a conversation would each pay for a TLS
// handshake. The cache is per run, not global: an environment change between
// runs is picked up, and no key outlives the flow that used it.

use std::collections::HashMap;

use super::client::Client;
use super::error::Error;
use super::spec::ProviderSpec;

/// The clients one run has built.
#[derive(Debug, Default)]
pub(crate) struct Cache {
    clients: HashMap<ProviderSpec, Client>,
}

impl Cache {
    /// The client for a spec, built on first use from the environment.
    pub(crate) fn get(&mut self, spec: &ProviderSpec) -> Result<Client, Error> {
        if let Some(client) = self.clients.get(spec) {
            return Ok(client.clone());
        }
        let client = Client::from_env(spec.clone())?;
        self.clients.insert(spec.clone(), client.clone());
        Ok(client)
    }
}
