// --- store::grant ---
// `hob trust`: decide which project may run its commands.
//
// The verb only ever acts on the project the working directory is in, so there
// is no path to typo and no way to trust the wrong checkout.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::cli::USAGE_EXIT;
use crate::cli::report::{self, Format};
use crate::effect::Failure;
use crate::paths::Paths;

use super::trust::Trusted;

/// Which management action the call asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Trust the project.
    Add,
    /// Revoke the project.
    Revoke,
    /// Print every trusted root.
    List,
}

/// `hob trust`, `--revoke`, `--list`.
pub(crate) fn trust(rest: &[String], out: &mut dyn Write, format: Format) -> Result<(), Failure> {
    let mut action = Action::Add;
    for arg in rest {
        match arg.as_str() {
            "--revoke" => action = pick(action, Action::Revoke, arg)?,
            "--list" => action = pick(action, Action::List, arg)?,
            _ => {
                return Err(Failure::with_code(
                    format!("unknown argument `{arg}`"),
                    USAGE_EXIT,
                ));
            }
        }
    }
    let mut trusted = Trusted::load()?;
    match action {
        Action::List => list(&trusted, out, format),
        Action::Add => changed(&trusted.add(&project()?)?, "trusted", out, format),
        Action::Revoke => changed(&trusted.remove(&project()?)?, "revoked", out, format),
    }
    Ok(())
}

/// Report one change: a line, or one object under `--json`.
fn changed(root: &Path, what: &str, out: &mut dyn Write, format: Format) {
    match format {
        Format::Text => {
            let _ = writeln!(out, "{what} {}", root.display());
        }
        Format::Json => {
            let value = Map::from_iter([(what.to_owned(), json!(root))]);
            report::json(out, &Value::Object(value));
        }
    }
}

/// Merge a flag into the action, rejecting contradictions.
fn pick(current: Action, wanted: Action, flag: &str) -> Result<Action, Failure> {
    if current == Action::Add || current == wanted {
        return Ok(wanted);
    }
    let other = if current == Action::Revoke {
        "--revoke"
    } else {
        "--list"
    };
    Err(Failure::with_code(
        format!("`{flag}` conflicts with `{other}`"),
        USAGE_EXIT,
    ))
}

/// The project the working directory is in.
fn project() -> Result<PathBuf, Failure> {
    Paths::resolve()
        .project_root()
        .map(PathBuf::from)
        .ok_or_else(|| {
            Failure::new("no project here (no `.hob` or `.git` above the working directory)")
        })
}

/// Print the store, one root per line, or as one array under `--json`.
fn list(trusted: &Trusted, out: &mut dyn Write, format: Format) {
    if format == Format::Json {
        report::json(out, &json!({ "trusted": trusted.roots() }));
        return;
    }
    if trusted.roots().is_empty() {
        let _ = writeln!(out, "no trusted projects");
    }
    for root in trusted.roots() {
        let _ = writeln!(out, "{}", root.display());
    }
}
