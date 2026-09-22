// --- store::query ---
// The reading verbs: list and which. Neither touches a file; they only render
// what discovery already found.

use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use crate::effect::Failure;

use super::suggest::suggest;
use super::{Command, Listing, one};

/// Print the effective commands and the files discovery ignored.
pub(crate) fn list(listing: &Listing, rest: &[String], out: &mut dyn Write) -> Result<(), Failure> {
    if let Some(arg) = rest.first() {
        return Err(Failure::with_code(
            format!("`list` takes no arguments, got `{arg}`"),
            crate::cli::USAGE_EXIT,
        ));
    }
    let commands = listing.effective_all();
    if commands.is_empty() {
        let _ = writeln!(out, "no commands found; create one with `hob new <name>`");
    } else {
        let mut name_width = "NAME".len();
        let mut origin_width = "ORIGIN".len();
        let mut rows = Vec::new();
        for command in commands {
            let (name, origin) = display(listing, command);
            name_width = name_width.max(name.len());
            origin_width = origin_width.max(origin.len());
            let summary = command.summary.clone().unwrap_or_else(|| "-".to_owned());
            rows.push((name, origin, summary));
        }
        let _ = writeln!(
            out,
            "{:<name_width$}  {:<origin_width$}  SUMMARY",
            "NAME", "ORIGIN"
        );
        for (name, origin, summary) in rows {
            let _ = writeln!(
                out,
                "{name:<name_width$}  {origin:<origin_width$}  {summary}"
            );
        }
    }
    for ignored in &listing.ignored {
        let _ = writeln!(
            out,
            "ignored: {} ({})",
            ignored.path.display(),
            ignored.reason
        );
    }
    Ok(())
}

/// Print every layer that defines a name, effective first.
pub(crate) fn which(
    listing: &Listing,
    rest: &[String],
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let name = one("which", rest)?;
    let chain: Vec<&Command> = listing.chain(name).collect();
    if chain.is_empty() {
        return Err(Failure::new(unknown(name, listing)));
    }
    for (index, command) in chain.iter().enumerate() {
        let status = if index == 0 { "effective" } else { "shadowed" };
        let _ = writeln!(
            out,
            "{status:<9}  {:<7}  {}",
            command.origin.label(),
            command.path.display()
        );
    }
    Ok(())
}

/// The message for a name that resolves to nothing, with a suggestion.
pub(crate) fn unknown(name: &str, listing: &Listing) -> String {
    let mut message = format!("unknown command `{name}`");
    let lua_file = Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lua"));
    if lua_file || name.contains('/') {
        let _ = write!(message, "\n  to run a file, use `hob run {name}`");
        return message;
    }
    let commands = listing.effective_all();
    let names = commands.iter().map(|command| command.name.as_str());
    let close = suggest(name, names);
    if !close.is_empty() {
        let list = close
            .iter()
            .map(|candidate| format!("`{candidate}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = write!(message, "\n  did you mean {list}?");
    }
    message
}

/// Name and origin column for one row; a project command that shadows a user
/// file says so.
fn display(listing: &Listing, command: &Command) -> (String, String) {
    let shadowed = listing.chain(&command.name).count() > 1;
    let origin = if shadowed {
        format!("{} (shadows user)", command.origin.label())
    } else {
        command.origin.label().to_owned()
    };
    (command.name.clone(), origin)
}
