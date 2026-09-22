// --- cli ---
// Argument parsing and top-level dispatch.

use std::io::Write;

use crate::paths::Paths;
use crate::store;

mod flow;
mod help;

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
    match args.next().as_ref().map(AsRef::as_ref) {
        None | Some("-h" | "--help") => flow::write(out, &help::text()),
        Some("-V" | "--version") => flow::write(out, &format!("{NAME} {VERSION}\n")),
        Some("run") => flow::file(&collect(args), err),
        Some("list") => {
            let listing = listing();
            flow::finish(store::list(&listing, &collect(args), out), err)
        }
        Some("new") => {
            let paths = Paths::resolve();
            let listing = store::scan(&paths);
            flow::finish(store::new(&paths, &listing, &collect(args), out), err)
        }
        Some("rm") => flow::finish(store::remove(&listing(), &collect(args), out), err),
        Some("which") => flow::finish(store::which(&listing(), &collect(args), out), err),
        Some(other) => dispatch(other, args, err),
    }
}

/// A bare word is a command name; a flag here is a mistake.
fn dispatch<I, S>(name: &str, args: I, err: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    if name.starts_with('-') {
        return flow::fail(
            err,
            &format!("unknown argument `{name}`\n\n{}", help::text()),
        );
    }
    let listing = listing();
    match listing.effective(name) {
        Some(command) => flow::command(command, &collect(args), err),
        None => flow::fail(err, &store::unknown(name, &listing)),
    }
}

/// The registry as the working directory and environment describe it.
fn listing() -> store::Listing {
    store::scan(&Paths::resolve())
}

fn collect<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect()
}
