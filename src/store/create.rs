// --- store::create ---
// `hob new`: write a command template into the project or the user layer.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::USAGE_EXIT;
use crate::effect::Failure;
use crate::paths::Paths;

use super::{Listing, RESERVED, valid_name};

/// Where `new` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    /// Project when there is one, the user's configuration otherwise.
    Auto,
    /// The user's configuration.
    User,
    /// The project, or the working directory outside one.
    Local,
}

/// Create one command file from a template.
pub(crate) fn new(
    paths: &Paths,
    listing: &Listing,
    rest: &[String],
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let mut name = None;
    let mut target = Target::Auto;
    for arg in rest {
        match arg.as_str() {
            "--user" => target = choose(target, Target::User)?,
            "--local" => target = choose(target, Target::Local)?,
            _ if arg.starts_with('-') => {
                return Err(Failure::with_code(
                    format!("unknown option `{arg}`"),
                    USAGE_EXIT,
                ));
            }
            _ if name.is_none() => name = Some(arg.clone()),
            _ => {
                return Err(Failure::with_code(
                    format!("unexpected argument `{arg}`"),
                    USAGE_EXIT,
                ));
            }
        }
    }
    let Some(name) = name else {
        return Err(Failure::with_code("`new` needs a name", USAGE_EXIT));
    };
    check(&name)?;
    let dir = directory(paths, target)?;
    let path = dir.join(format!("{name}.lua"));
    if path.exists() {
        return Err(Failure::new(format!(
            "`{name}` already exists at {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            Failure::new(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    fs::write(&path, template(&name))
        .map_err(|error| Failure::new(format!("cannot write {}: {error}", path.display())))?;
    let _ = writeln!(out, "created {}", path.display());
    if let Some(shadow) = listing.effective(&name) {
        let _ = writeln!(
            out,
            "note: `{name}` is shadowed by {}",
            shadow.path.display()
        );
    }
    Ok(())
}

/// Merge a flag into the target, rejecting contradictions.
fn choose(current: Target, wanted: Target) -> Result<Target, Failure> {
    if current == wanted {
        return Ok(current);
    }
    if current != Target::Auto {
        return Err(Failure::with_code(
            "`--user` and `--local` are mutually exclusive",
            USAGE_EXIT,
        ));
    }
    Ok(wanted)
}

/// The directory the file goes into.
fn directory(paths: &Paths, target: Target) -> Result<PathBuf, Failure> {
    match target {
        Target::User => paths.user_commands(),
        Target::Local => Some(paths.local_commands()),
        Target::Auto => paths.project_commands().or_else(|| paths.user_commands()),
    }
    .ok_or_else(|| {
        Failure::new("cannot determine the configuration directory (set HOME or HOB_CONFIG_DIR)")
    })
}

/// Reject names that discovery would not accept.
fn check(name: &str) -> Result<(), Failure> {
    if !valid_name(name) {
        return Err(Failure::new(format!(
            "`{name}` is not a valid command name \
             (expected [a-z][a-z0-9-]* segments joined by `/`, 32 characters at most)"
        )));
    }
    if RESERVED.contains(&name) {
        return Err(Failure::new(format!("`{name}` is reserved for the CLI")));
    }
    Ok(())
}

/// The starting point `hob new` writes.
fn template(name: &str) -> String {
    format!("--- TODO: summarize {name}.\n\nhob.term.print(\"hello from \" .. hob.command)\n")
}
