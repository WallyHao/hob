// --- builtin::report ---
// The two shapes of `hob doctor`: lines for a person, one object for a script.

use std::io::Write;
use std::path::Path;

use serde_json::{Value, json};

use crate::cli::{NAME, VERSION};
use crate::paths::Paths;
use crate::provider;
use crate::store::trust::Trusted;
use crate::store::{Listing, Origin};

/// One key variable the doctor reports on, by name and never by value.
struct Key {
    id: String,
    env: String,
    set: bool,
}

/// The text form.
pub(super) fn text(paths: &Paths, listing: &Listing, out: &mut dyn Write) {
    let (project, user, builtin) = counts(listing);
    let _ = writeln!(out, "{NAME} {VERSION}");
    let _ = writeln!(out, "config:  {}", root(paths.config_root()));
    let _ = writeln!(out, "project: {}", root(paths.project_root()));
    let _ = writeln!(
        out,
        "commands: {} effective ({project} project, {user} user, {builtin} builtin)",
        listing.effective_all().len()
    );
    let _ = writeln!(out, "trust:   {}", trust(paths));
    let _ = writeln!(out, "keys:    {}", keys());
}

/// The `--json` form: the same facts as one object.
pub(super) fn json(paths: &Paths, listing: &Listing, out: &mut dyn Write) {
    let (project, user, builtin) = counts(listing);
    let trust = match Trusted::load() {
        Ok(trusted) => json!({
            "project": paths.project_root(),
            "trusted": paths.project_root().is_some_and(|root| trusted.allows(root)),
        }),
        Err(error) => json!({ "project": paths.project_root(), "error": error.message }),
    };
    let keys = match known_keys() {
        Ok(keys) => Value::Array(
            keys.iter()
                .map(|key| json!({ "id": key.id, "env": key.env, "set": key.set }))
                .collect(),
        ),
        Err(error) => json!({ "error": error }),
    };
    let value = json!({
        "version": format!("{NAME} {VERSION}"),
        "config": paths.config_root(),
        "project": paths.project_root(),
        "commands": {
            "effective": listing.effective_all().len(),
            "project": project,
            "user": user,
            "builtin": builtin,
        },
        "trust": trust,
        "keys": keys,
    });
    let _ = writeln!(out, "{value:#}");
}

/// A root, or a word for a layer that is not there.
fn root(root: Option<&Path>) -> String {
    root.map_or_else(|| "none".to_owned(), |path| path.display().to_string())
}

/// Effective commands per layer.
fn counts(listing: &Listing) -> (usize, usize, usize) {
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

/// Whether the working directory's project may run its commands.
fn trust(paths: &Paths) -> String {
    let trusted = match Trusted::load() {
        Ok(trusted) => trusted,
        Err(error) => return format!("cannot read the trust store: {}", error.message),
    };
    let Some(root) = paths.project_root() else {
        return "no project".to_owned();
    };
    let state = if trusted.allows(root) {
        "trusted"
    } else {
        "untrusted"
    };
    format!("{} ({state})", root.display())
}

/// Which key variables are set, by name and never by value.
fn keys() -> String {
    match known_keys() {
        Ok(keys) if keys.is_empty() => "none".to_owned(),
        Ok(keys) => keys
            .iter()
            .map(|key| {
                let state = if key.set { "set" } else { "unset" };
                format!("{} ({} {state})", key.id, key.env)
            })
            .collect::<Vec<_>>()
            .join(", "),
        Err(error) => format!("cannot read the configuration: {error}"),
    }
}

/// The key variables, once, so both shapes agree on what was looked up.
fn known_keys() -> Result<Vec<Key>, String> {
    let providers = provider::known().map_err(|error| error.to_string())?;
    Ok(providers
        .iter()
        .map(|spec| Key {
            id: spec.id.clone(),
            env: spec.api_key_env.clone(),
            set: std::env::var_os(&spec.api_key_env).is_some(),
        })
        .collect())
}
