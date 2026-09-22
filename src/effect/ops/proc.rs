// --- effect::ops::proc ---
// Arguments of the `proc` namespace.

use std::collections::BTreeMap;

use serde::Deserialize;

/// `proc.open`: the context a session starts in.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Open {
    /// Working directory; the current one when absent.
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    /// Environment overrides on top of the process environment.
    #[serde(default)]
    pub(crate) env: BTreeMap<String, String>,
    /// Shell text evaluated before each shell line.
    #[serde(default)]
    pub(crate) profile: Option<String>,
}

/// Options shared by `exec` and `shell`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Options {
    /// Session handle; absent means a one-shot command in a fresh context.
    #[serde(default)]
    pub(crate) session: Option<u64>,
    /// Hand the terminal over instead of capturing output.
    #[serde(default)]
    pub(crate) inherit: bool,
    /// Text written to the child's standard input.
    #[serde(default)]
    pub(crate) stdin: Option<String>,
    /// Kill the child after this many milliseconds; the code is then 124.
    #[serde(default)]
    pub(crate) timeout_ms: Option<u64>,
    /// Trim trailing whitespace from the captured streams.
    #[serde(default)]
    pub(crate) trim: bool,
}

/// `proc.exec`.
#[derive(Debug, Deserialize)]
pub(crate) struct Exec {
    /// Program and arguments.
    pub(crate) argv: Vec<String>,
    #[serde(flatten)]
    pub(crate) options: Options,
}

/// `proc.shell`.
#[derive(Debug, Deserialize)]
pub(crate) struct Shell {
    /// One shell line, run through `sh -c`.
    pub(crate) line: String,
    #[serde(flatten)]
    pub(crate) options: Options,
}

/// `proc.which`.
#[derive(Debug, Deserialize)]
pub(crate) struct Which {
    /// Program name to resolve on `PATH`.
    pub(crate) prog: String,
}

/// A handle to one session, for the methods that need nothing else.
#[derive(Debug, Deserialize)]
pub(crate) struct Handle {
    /// Session handle returned by `proc.open`.
    pub(crate) session: u64,
}

/// `proc.setenv`.
#[derive(Debug, Deserialize)]
pub(crate) struct SetEnv {
    pub(crate) session: u64,
    pub(crate) name: String,
    pub(crate) value: String,
}

/// `proc.unset`.
#[derive(Debug, Deserialize)]
pub(crate) struct Unset {
    pub(crate) session: u64,
    pub(crate) name: String,
}

/// `proc.chdir`.
#[derive(Debug, Deserialize)]
pub(crate) struct Chdir {
    pub(crate) session: u64,
    pub(crate) path: String,
}

/// `proc.setup`.
#[derive(Debug, Deserialize)]
pub(crate) struct Setup {
    pub(crate) session: u64,
    pub(crate) text: String,
}
