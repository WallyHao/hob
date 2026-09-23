// --- store::libraries ---
// Resolve code search roots before execution, so requiring a helper cannot
// bypass the project trust boundary through an otherwise trusted user command.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::effect::Failure;
use crate::paths::Paths;

use super::trust::Trusted;

/// Library roots authorized for one run, in precedence order.
#[derive(Debug)]
pub(crate) struct Libraries {
    dirs: Vec<PathBuf>,
}

impl Libraries {
    /// An explicit flow authorizes its entry, not the surrounding project's code.
    pub(crate) fn resolve(paths: &Paths) -> Result<Self, Failure> {
        let mut dirs = Vec::new();
        if let (Some(root), Some(path)) = (paths.project_root(), paths.project_libs())
            && Trusted::load()?.allows(root)
        {
            add(&mut dirs, root, &path)?;
        }
        if let (Some(root), Some(path)) = (paths.config_root(), paths.user_libs()) {
            add(&mut dirs, root, &path)?;
        }
        Ok(Self { dirs })
    }

    /// Return the checked target, never the unchecked symlink spelling.
    pub(crate) fn find(&self, relative: &Path) -> Result<Option<PathBuf>, Failure> {
        for root in &self.dirs {
            let candidate = root.join(relative);
            if !present(&candidate)? {
                continue;
            }
            let target = within(&candidate, root)?;
            if target.is_file() {
                return Ok(Some(target));
            }
        }
        Ok(None)
    }
}

/// Pin existing library roots; a symlink cannot expand its owner's authority.
fn add(dirs: &mut Vec<PathBuf>, owner: &Path, path: &Path) -> Result<(), Failure> {
    if present(path)? {
        let owner = canonical(owner)?;
        let root = within(path, &owner)?;
        if !root.is_dir() {
            return Err(Failure::new(format!(
                "library root `{}` is not a directory",
                path.display()
            )));
        }
        dirs.push(root);
    }
    Ok(())
}

/// Missing modules may fall back; dangling final links remain present and fail.
fn present(path: &Path) -> Result<bool, Failure> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Failure::new(format!(
            "cannot inspect library path `{}`: {error}",
            path.display()
        ))),
    }
}

fn canonical(path: &Path) -> Result<PathBuf, Failure> {
    fs::canonicalize(path).map_err(|error| {
        Failure::new(format!(
            "cannot resolve library path `{}`: {error}",
            path.display()
        ))
    })
}

fn within(path: &Path, boundary: &Path) -> Result<PathBuf, Failure> {
    let target = canonical(path)?;
    if !target.starts_with(boundary) {
        return Err(Failure::new(format!(
            "library path `{}` resolves outside authorized directory `{}`",
            path.display(),
            boundary.display()
        )));
    }
    Ok(target)
}
