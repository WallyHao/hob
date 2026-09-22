// --- exec ---
// Where effects are performed. Dispatch is by `ns.op` and nothing else, so an
// unimplemented operation is a named error rather than a silent no-op.

pub(crate) mod logs;
pub(crate) mod term;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::effect::{Failure, Request};

/// Perform one effect.
pub(crate) fn perform(request: &Request) -> Result<Value, Failure> {
    match (request.ns.as_str(), request.op.as_str()) {
        ("logs", "write") => logs::write(&decode(request)?),
        ("term", "print") => term::print(&decode(request)?),
        _ => Err(Failure::unknown(&request.ns, &request.op)),
    }
}

fn decode<T: DeserializeOwned>(request: &Request) -> Result<T, Failure> {
    serde_json::from_value(request.cmd.clone())
        .map_err(|error| Failure::new(format!("`{}.{}`: {error}", request.ns, request.op)))
}
