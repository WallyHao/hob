// --- store::trust ---
// The project roots whose commands may run.
//
// The store lives in the user's configuration, never in the project: a checkout
// that could bless its own commands would make the trust boundary meaningless.
// Roots are stored canonicalized, so a command reached through a symlink or a
// relative path compares equal to the one that was trusted.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::effect::Failure;
use crate::paths;

use super::{Command, Origin};

/// The project roots a user has trusted.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Trusted {
    #[serde(default)]
    projects: Vec<PathBuf>,
}

impl Trusted {
    /// Load the store; a missing file is an empty store.
    pub(crate) fn load() -> Result<Self, Failure> {
        let Some(path) = file() else {
            return Ok(Self::default());
        };
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(Failure::new(format!(
                    "cannot read `{}`: {error}",
                    path.display()
                )));
            }
        };
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str(&text)
            .map_err(|error| Failure::new(format!("cannot parse `{}`: {error}", path.display())))
    }

    /// Whether a project root may run its commands.
    pub(crate) fn allows(&self, project: &Path) -> bool {
        canonical(project).is_ok_and(|root| self.projects.contains(&root))
    }

    /// Whether the project a command file lives in may run it.
    pub(crate) fn allows_file(&self, file: &Path) -> bool {
        paths::project_root(file).is_some_and(|root| self.allows(&root))
    }

    /// Trust a root, returning its canonical form.
    pub(crate) fn add(&mut self, path: &Path) -> Result<PathBuf, Failure> {
        let root = canonical(path)?;
        if !self.projects.contains(&root) {
            self.projects.push(root.clone());
            self.save()?;
        }
        Ok(root)
    }

    /// Revoke a root, returning its canonical form.
    pub(crate) fn remove(&mut self, path: &Path) -> Result<PathBuf, Failure> {
        let root = canonical(path)?;
        if self.projects.contains(&root) {
            self.projects.retain(|known| known != &root);
            self.save()?;
        }
        Ok(root)
    }

    /// The roots, in the order they were added.
    pub(crate) fn roots(&self) -> &[PathBuf] {
        &self.projects
    }

    fn save(&self) -> Result<(), Failure> {
        let path = file().ok_or_else(|| {
            Failure::new("cannot find the configuration directory (set HOME or HOB_CONFIG_DIR)")
        })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Failure::new(format!("cannot create `{}`: {error}", parent.display()))
            })?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|error| Failure::new(format!("cannot encode the trust store: {error}")))?;
        // Write, then rename: an interrupted run cannot leave a half file, and
        // the pid keeps two runs from sharing a temporary.
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&tmp, text)
            .map_err(|error| Failure::new(format!("cannot write `{}`: {error}", tmp.display())))?;
        fs::rename(&tmp, &path)
            .map_err(|error| Failure::new(format!("cannot replace `{}`: {error}", path.display())))
    }
}

/// Why a command may not run, when its project is untrusted.
pub(crate) fn denial(command: &Command) -> Option<Failure> {
    if command.origin != Origin::Project {
        return None;
    }
    let path = command.path()?;
    match Trusted::load() {
        Ok(trusted) if trusted.allows_file(path) => None,
        Ok(_) => Some(Failure::new(format!(
            "project `{}` is not trusted\n  run `hob trust` from it to allow its commands",
            label(path)
        ))),
        Err(failure) => Some(failure),
    }
}

/// The project root a command file belongs to, for a message.
fn label(path: &Path) -> String {
    paths::project_root(path).map_or_else(
        || path.display().to_string(),
        |root| root.display().to_string(),
    )
}

/// The trust file in the user's configuration, when there is one.
fn file() -> Option<PathBuf> {
    paths::config_dir().map(|dir| dir.join("trust.json"))
}

/// A canonical path, or a failure naming the input.
fn canonical(path: &Path) -> Result<PathBuf, Failure> {
    fs::canonicalize(path)
        .map_err(|error| Failure::new(format!("cannot resolve `{}`: {error}", path.display())))
}
