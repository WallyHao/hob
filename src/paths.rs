// --- paths ---
// Every directory hob reads, resolved in one place.
//
// `HOB_CONFIG_DIR` wins over the XDG location so tests and throwaway profiles
// run without touching the real configuration. The project root is the nearest
// ancestor of the working directory holding `.hob` or `.git`, the way git finds
// a repository, so a command works from any subdirectory of a checkout.

use std::env;
use std::path::{Path, PathBuf};

/// Resolved locations of the user and project layers.
#[derive(Debug, Clone)]
pub(crate) struct Paths {
    /// User configuration root, e.g. `~/.config/hob`.
    config: Option<PathBuf>,
    /// Nearest ancestor holding `.hob` or `.git`.
    project: Option<PathBuf>,
}

impl Paths {
    /// Resolve from the environment and the working directory.
    pub(crate) fn resolve() -> Self {
        Self {
            config: config_dir(),
            project: env::current_dir().ok().and_then(|cwd| project_root(&cwd)),
        }
    }

    /// `commands/` under the user's configuration, when it can be found.
    pub(crate) fn user_commands(&self) -> Option<PathBuf> {
        self.config.as_ref().map(|dir| dir.join("commands"))
    }

    /// `commands/` under the project root, when there is one.
    pub(crate) fn project_commands(&self) -> Option<PathBuf> {
        self.project
            .as_ref()
            .map(|root| root.join(".hob").join("commands"))
    }

    /// Where `new --local` writes: the project root, or the working directory
    /// when the command is not inside a project.
    pub(crate) fn local_commands(&self) -> PathBuf {
        let root = self
            .project
            .clone()
            .or_else(|| env::current_dir().ok())
            .unwrap_or_default();
        root.join(".hob").join("commands")
    }

    /// `prompts/` under the project root, when there is one.
    pub(crate) fn project_prompts(&self) -> Option<PathBuf> {
        self.project
            .as_ref()
            .map(|root| root.join(".hob").join("prompts"))
    }

    /// `prompts/` under the user's configuration, when it can be found.
    pub(crate) fn user_prompts(&self) -> Option<PathBuf> {
        self.config.as_ref().map(|dir| dir.join("prompts"))
    }
}

/// `$HOB_CONFIG_DIR`, else `$XDG_CONFIG_HOME/hob`, else `~/.config/hob`.
pub(crate) fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("HOB_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("hob"))
}

/// Walk up until a directory holds `.hob` or `.git`.
fn project_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|dir| dir.join(".hob").exists() || dir.join(".git").exists())
        .map(Path::to_path_buf)
}
