// --- cli ---
// Argument parsing and top-level dispatch.

use std::io::Write;

use crate::driver::Control;
use crate::paths::Paths;
use crate::store;

mod flags;
mod flow;
mod help;
pub(crate) mod report;
use crate::trace;

use report::Format;

/// Name of the binary, taken from the manifest so it is spelled once.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Version of the binary, taken from the manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit code for a command line that could not be understood.
pub const USAGE_EXIT: u8 = 2;

/// Run one command line. `out` and `err` are injected so tests and benchmarks
/// can capture output without touching the process's real streams.
pub fn run<I, S>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    let first = args.next();
    // The two fixed answers skip the parser: they are what a wrapper script runs
    // most, and there is no flag they could combine with.
    match first.as_ref().map(AsRef::as_ref) {
        Some("-h" | "--help") => return report::text(out, help::text()),
        Some("-V" | "--version") => return report::text(out, &format!("{NAME} {VERSION}\n")),
        _ => {}
    }
    let invocation = match flags::parse(first.into_iter().chain(args)) {
        Ok(invocation) => invocation,
        // The flags did not parse, so `--json` is not known to be on: text it is.
        Err(message) => {
            return report::error(
                err,
                &format!("{message}\n\n{}", help::text()),
                USAGE_EXIT,
                Format::Text,
            );
        }
    };
    match invocation {
        flags::Invocation::Help => report::text(out, help::text()),
        flags::Invocation::Version => report::text(out, &format!("{NAME} {VERSION}\n")),
        flags::Invocation::Command {
            control,
            trace,
            format,
            words,
        } => {
            if words.is_empty() {
                return report::text(out, help::text());
            }
            dispatch(&words, control, trace, format, out, err)
        }
    }
}

/// Run the verb or command the first word names.
fn dispatch(
    words: &[String],
    control: Control,
    trace: Option<trace::Settings>,
    format: Format,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let name = words[0].as_str();
    let rest = &words[1..];
    match name {
        "run" => flow::file(rest, control, trace, format, err),
        "list" => {
            let listing = listing();
            let result = if format == Format::Json {
                report::list(&listing, rest, out)
            } else {
                store::list(&listing, rest, out)
            };
            report::finish(result, err, format)
        }
        "new" => {
            let paths = Paths::resolve();
            let listing = store::scan(&paths);
            report::finish(store::new(&paths, &listing, rest, out), err, format)
        }
        "rm" => report::finish(store::remove(&listing(), rest, out), err, format),
        "trust" => report::finish(store::trust(rest, out, format), err, format),
        "which" => {
            let listing = listing();
            let result = if format == Format::Json {
                report::which(&listing, rest, out)
            } else {
                store::which(&listing, rest, out)
            };
            report::finish(result, err, format)
        }
        _ => {
            let listing = listing();
            match listing.effective(name) {
                Some(command) => flow::command(command, rest, control, trace, format, out, err),
                None => report::error(err, &store::unknown(name, &listing), USAGE_EXIT, format),
            }
        }
    }
}

/// The registry as the working directory and environment describe it.
fn listing() -> store::Listing {
    store::scan(&Paths::resolve())
}
