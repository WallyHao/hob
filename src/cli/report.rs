// --- cli::report ---
// Machine-readable answers for the verbs that describe the installation.
//
// The text forms stay with the store; this module owns the `--json` shape, so
// a script gets one object per run and a new field never breaks the old ones.

use std::io::Write;

use serde_json::{Value, json};

use crate::effect::Failure;
use crate::store::trust::Trusted;
use crate::store::{self, Command, Listing, Origin};

/// The shape a describing verb prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Format {
    /// Text for a person at a terminal.
    #[default]
    Text,
    /// One JSON object for a script.
    Json,
}

/// `hob list --json`: effective commands, then the files discovery ignored.
pub(crate) fn list(listing: &Listing, rest: &[String], out: &mut dyn Write) -> Result<(), Failure> {
    if let Some(arg) = rest.first() {
        return Err(Failure::with_code(
            format!("`list` takes no arguments, got `{arg}`"),
            crate::cli::USAGE_EXIT,
        ));
    }
    let trusted = Trusted::load().unwrap_or_default();
    let commands: Vec<Value> = listing
        .effective_all()
        .into_iter()
        .map(|command| record(listing, command, &trusted))
        .collect();
    let ignored: Vec<Value> = listing
        .ignored
        .iter()
        .map(|entry| json!({ "path": entry.path, "reason": entry.reason }))
        .collect();
    json(out, &json!({ "commands": commands, "ignored": ignored }));
    Ok(())
}

/// `hob which --json`: every layer that defines a name, effective first.
pub(crate) fn which(
    listing: &Listing,
    rest: &[String],
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let name = store::one("which", rest)?;
    let layers: Vec<Value> = listing
        .chain(name)
        .enumerate()
        .map(|(index, command)| {
            json!({
                "origin": command.origin.label(),
                "path": command.path(),
                "effective": index == 0,
            })
        })
        .collect();
    if layers.is_empty() {
        return Err(Failure::new(store::unknown(name, listing)));
    }
    json(out, &json!({ "name": name, "layers": layers }));
    Ok(())
}

/// Print a failure: text for a person, one object for a script.
pub(crate) fn error(err: &mut dyn Write, message: &str, code: u8, format: Format) -> u8 {
    if message.is_empty() {
        return code;
    }
    match format {
        Format::Text => {
            let _ = writeln!(err, "error: {message}");
        }
        Format::Json => json(err, &json!({ "error": message })),
    }
    code
}

/// Print the outcome of a verb that returns nothing.
pub(crate) fn finish(result: Result<(), Failure>, err: &mut dyn Write, format: Format) -> u8 {
    match result {
        Ok(()) => 0,
        Err(failure) => error(err, &failure.message, failure.code, format),
    }
}

/// Print raw text, or 1 when the stream is gone.
pub(crate) fn text(out: &mut dyn Write, text: &str) -> u8 {
    u8::from(out.write_all(text.as_bytes()).is_err())
}

/// Print one JSON object, pretty enough to read and to pipe.
pub(crate) fn json(out: &mut dyn Write, value: &Value) {
    let _ = writeln!(out, "{value:#}");
}

/// One `list` row.
fn record(listing: &Listing, command: &Command, trusted: &Trusted) -> Value {
    json!({
        "name": command.name.as_str(),
        "origin": command.origin.label(),
        "path": command.path(),
        "summary": command.meta.summary.as_deref(),
        "args": command.meta.args.map(store::Args::describe),
        "shadowed": listing.chain(&command.name).count() > 1,
        "trusted": command.origin != Origin::Project
            || command.path().is_some_and(|path| trusted.allows_file(path)),
    })
}
