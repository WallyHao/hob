// --- builtin::doctor ---
// `hob doctor`: what the installation looks like from the inside.
//
// It reports paths and counts, and whether each provider's key variable is set,
// by name only: a doctor that printed a value would be a leak with a friendly
// face.

use std::io::Write;
use std::path::Path;

use crate::effect::Failure;
use crate::paths::Paths;
use crate::provider;
use crate::store::{self, Args, Command, Meta, Origin, Source};

/// The builtin itself.
pub(crate) fn command() -> Command {
    Command {
        name: "doctor".to_owned(),
        origin: Origin::Builtin,
        source: Source::Builtin,
        meta: Meta {
            summary: Some("Report the configuration, the project and the keys.".to_owned()),
            usage: Some("doctor".to_owned()),
            args: Args::parse("0"),
        },
    }
}

/// Print the state of the installation.
// The builtin table stays uniform: every entry returns a Result so the next one
// may fail, and `builtin::run` has one shape to map.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn run(out: &mut dyn Write) -> Result<(), Failure> {
    let paths = Paths::resolve();
    let listing = store::scan(&paths);
    let (project, user, builtin) = counts(&listing);
    let _ = writeln!(out, "{} {}", crate::cli::NAME, crate::cli::VERSION);
    let _ = writeln!(out, "config:  {}", root(paths.config_root()));
    let _ = writeln!(out, "project: {}", root(paths.project_root()));
    let _ = writeln!(
        out,
        "commands: {} effective ({project} project, {user} user, {builtin} builtin)",
        listing.effective_all().len()
    );
    let _ = writeln!(out, "keys:    {}", keys());
    Ok(())
}

/// A root, or a word for a layer that is not there.
fn root(root: Option<&Path>) -> String {
    root.map_or_else(|| "none".to_owned(), |path| path.display().to_string())
}

/// Effective commands per layer.
fn counts(listing: &store::Listing) -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for command in listing.effective_all() {
        match command.origin {
            Origin::Project => counts.0 += 1,
            Origin::User => counts.1 += 1,
            Origin::Builtin => counts.2 += 1,
        }
    }
    counts
}

/// Which key variables are set, by name and never by value.
fn keys() -> String {
    let providers = match provider::known() {
        Ok(providers) => providers,
        Err(error) => return format!("cannot read the configuration: {error}"),
    };
    if providers.is_empty() {
        return "none".to_owned();
    }
    providers
        .iter()
        .map(|spec| {
            let state = if std::env::var_os(&spec.api_key_env).is_some() {
                "set"
            } else {
                "unset"
            };
            format!("{} ({} {state})", spec.id, spec.api_key_env)
        })
        .collect::<Vec<_>>()
        .join(", ")
}
