// --- effect::ops::tmpl ---
// Arguments of the `tmpl` namespace.

use serde::Deserialize;

/// `tmpl.fetch`.
#[derive(Debug, Deserialize)]
pub(crate) struct Fetch {
    /// Template name, a relative path under a prompts directory.
    pub(crate) name: String,
}
