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

/// What a session started with.
#[derive(Debug, Clone)]
struct Opening {
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    profile: Option<String>,
}

/// A session opened by `proc.open`.
#[derive(Debug)]
pub(crate) struct Session {
    opening: Opening,
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    profile: Option<String>,
}

impl Session {
    /// Open a session in `cwd`, with `env` overrides and an optional profile.
    pub(crate) fn open(
        cwd: Option<String>,
        env: BTreeMap<String, String>,
        profile: Option<String>,
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
            cwd: cwd.clone(),
            env: env.clone(),
            profile: profile.clone(),
        };
        Ok(Self {
            opening,
            cwd,
            env,
            profile,
        })
    }

    /// Where commands run.
    pub(crate) fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Environment overrides, on top of the process environment.
    pub(crate) fn env(&self) -> &BTreeMap<String, String> {
        &self.env
    }

    /// Shell text evaluated before each shell line.
    pub(crate) fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Move the session; a relative path is joined to the current one.
    pub(crate) fn chdir(&mut self, path: &str) -> Result<(), Failure> {
        let dir = self.cwd.join(path);
        if !dir.is_dir() {
            return Err(Failure::new(format!(
                "`{}` is not a directory",
                dir.display()
            )));
        }
        self.cwd = dir;
        Ok(())
    }

    /// Add or replace one environment override.
    pub(crate) fn setenv(&mut self, name: &str, value: &str) {
        self.env.insert(name.to_owned(), value.to_owned());
    }

    /// Remove one override, exposing the process value again.
    pub(crate) fn unset(&mut self, name: &str) {
        self.env.remove(name);
    }

    /// Replace the profile.
    pub(crate) fn setup(&mut self, text: &str) {
        self.profile = Some(text.to_owned());
    }

    /// Restore the context the session opened with.
    pub(crate) fn reset(&mut self) {
        self.cwd.clone_from(&self.opening.cwd);
        self.env.clone_from(&self.opening.env);
        self.profile.clone_from(&self.opening.profile);
    }

    /// What the session looks like now.
    pub(crate) fn state(&self) -> Value {
        json!({ "cwd": self.cwd, "env": self.env })
    }
}
