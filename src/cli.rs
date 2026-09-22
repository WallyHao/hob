//! Argument parsing and top-level dispatch.

use std::io::Write;

/// Name of the binary, taken from the manifest so it is spelled once.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Version of the binary, taken from the manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit code for a command line that could not be understood.
pub const USAGE_EXIT: u8 = 2;

enum Invocation {
    Help,
    Version,
    Unknown(String),
}

/// Run one command line. `out` and `err` are injected so tests and benchmarks
/// can capture output without touching the process's real streams.
pub fn run<I, S>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match classify(args) {
        Invocation::Help => write(out, &help()),
        Invocation::Version => write(out, &format!("{NAME} {VERSION}\n")),
        Invocation::Unknown(argument) => write(
            err,
            &format!("error: unknown argument `{argument}`\n\n{}", help()),
        )
        .max(USAGE_EXIT),
    }
}

fn classify<I, S>(args: I) -> Invocation
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    match args.next().as_ref().map(AsRef::as_ref) {
        None | Some("-h" | "--help") => Invocation::Help,
        Some("-V" | "--version") => Invocation::Version,
        Some(other) => Invocation::Unknown(other.to_owned()),
    }
}

fn write(target: &mut dyn Write, text: &str) -> u8 {
    u8::from(target.write_all(text.as_bytes()).is_err())
}

fn help() -> String {
    format!(
        "{NAME} {VERSION}\n\
         Scriptable CLI for AI chores.\n\
         \n\
         Usage: {NAME} <command> [options]\n\
         \n\
         Commands:\n\
         \n\
         Options:\n\
           -h, --help     Print help\n\
           -V, --version  Print version\n"
    )
}
