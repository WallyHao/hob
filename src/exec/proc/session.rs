// --- exec::proc::session ---
// One shell-like context: where commands run, what environment they see and
// what is evaluated before each shell line.
//
// The opening context is kept beside the current one so `reset` restores what
// the session started with rather than what the working directory happens to
// be at the time.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::effect::Failure;

/// A context a session can be in.
#[derive(Debug, Clone)]
struct Opening {
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    profile: Option<String>,
    env_clear: bool,
}

/// A session opened by `proc.open`.
#[derive(Debug)]
pub(crate) struct Session {
    opening: Opening,
    current: Opening,
}

impl Session {
    /// Open a session in `cwd`, with `env` overrides, an optional profile and
    /// whether commands start with an empty environment.
    pub(crate) fn open(
        cwd: Option<String>,
        env: BTreeMap<String, String>,
        profile: Option<String>,
        env_clear: bool,
    ) -> Result<Self, Failure> {
        let cwd = match cwd {
            Some(path) => PathBuf::from(path),
            None => std::env::current_dir().map_err(|error| {
                Failure::new(format!("cannot find the working directory: {error}"))
            })?,
        };
        if !cwd.is_dir() {
            return Err(Failure::new(format!(
                "`{}` is not a directory",
                cwd.display()
            )));
        }
        let opening = Opening {
            cwd,
            env,
            profile,
            env_clear,
        };
        Ok(Self {
            current: opening.clone(),
            opening,
        })
    }

    /// Where commands run.
    pub(crate) fn cwd(&self) -> &Path {
        &self.current.cwd
    }

    /// Environment overrides, on top of the process environment.
    pub(crate) fn env(&self) -> &BTreeMap<String, String> {
        &self.current.env
    }

    /// Shell text evaluated before each shell line.
    pub(crate) fn profile(&self) -> Option<&str> {
        self.current.profile.as_deref()
    }

    /// Whether commands start with an empty environment.
    pub(crate) fn env_clear(&self) -> bool {
        self.current.env_clear
    }

    /// Move the session; a relative path is joined to the current one.
    pub(crate) fn chdir(&mut self, path: &str) -> Result<(), Failure> {
        let dir = self.current.cwd.join(path);
        if !dir.is_dir() {
            return Err(Failure::new(format!(
                "`{}` is not a directory",
                dir.display()
            )));
        }
        self.current.cwd = dir;
        Ok(())
    }

    /// Add or replace one environment override.
    pub(crate) fn setenv(&mut self, name: &str, value: &str) {
        self.current.env.insert(name.to_owned(), value.to_owned());
    }

    /// Remove one override, exposing the process value again.
    pub(crate) fn unset(&mut self, name: &str) {
        self.current.env.remove(name);
    }

    /// Replace the profile.
    pub(crate) fn setup(&mut self, text: &str) {
        self.current.profile = Some(text.to_owned());
    }

    /// Restore the context the session opened with.
    pub(crate) fn reset(&mut self) {
        self.current.clone_from(&self.opening);
    }

    /// What the session looks like now.
    pub(crate) fn state(&self) -> Value {
        json!({
            "cwd": self.current.cwd,
            "env": self.current.env,
            "env_clear": self.current.env_clear,
        })
    }
}
