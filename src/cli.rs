//! Argument parsing and top-level dispatch.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::driver;

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
        None | Some("-h" | "--help") => write(out, &help()),
        Some("-V" | "--version") => write(out, &format!("{NAME} {VERSION}\n")),
        // Only `run` needs the rest of the arguments, and only it pays for
        // collecting them.
        Some("run") => {
            let rest: Vec<String> = args.map(|arg| arg.as_ref().to_owned()).collect();
            match rest.first() {
                Some(path) => execute(path, &rest[1..], err),
                None => write(
                    err,
                    &format!("error: `run` needs a flow file\n\n{}", help()),
                )
                .max(USAGE_EXIT),
            }
        }
        Some(other) => write(
            err,
            &format!("error: unknown argument `{other}`\n\n{}", help()),
        )
        .max(USAGE_EXIT),
    }
}

fn execute(path: &str, args: &[String], err: &mut dyn Write) -> u8 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return write(err, &format!("error: cannot read `{path}`: {error}\n")).max(1);
        }
    };
    let name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("flow");
    match driver::run(name, &source, args) {
        Ok(()) => 0,
        Err(failure) if failure.message.is_empty() => failure.code,
        Err(failure) => write(err, &format!("error: {}\n", failure.message)).max(failure.code),
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
           run <file.lua> [args...]  Run a flow\n\
         \n\
         Options:\n\
           -h, --help     Print help\n\
           -V, --version  Print version\n"
    )
}
