// --- effect::ops::file ---
// Arguments of the `file` namespace.

use serde::Deserialize;

/// `file.read`.
#[derive(Debug, Deserialize)]
pub(crate) struct Read {
    /// Path to read, relative to the working directory.
    pub(crate) path: String,
    /// Return `nil` instead of failing when the file does not exist.
    #[serde(default)]
    pub(crate) optional: bool,
}

/// `file.write`.
#[derive(Debug, Deserialize)]
pub(crate) struct Write {
    /// Path to write, parents created as needed.
    pub(crate) path: String,
    /// Text to write.
    pub(crate) text: String,
    /// Append instead of replacing.
    #[serde(default)]
    pub(crate) append: bool,
}

/// `file.stat`.
#[derive(Debug, Deserialize)]
pub(crate) struct Stat {
    /// Path to inspect.
    pub(crate) path: String,
}

/// `file.list`.
#[derive(Debug, Deserialize)]
pub(crate) struct List {
    /// Directory to enumerate.
    pub(crate) path: String,
}
