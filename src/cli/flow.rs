// --- cli::flow ---
// Running a flow, whether it came from a path (`hob run`) or from the
// registry (a bare command name), plus the reporting shared by every verb.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::driver::{self, Control};
use crate::effect::Failure;
use crate::store::Command;

use super::USAGE_EXIT;

/// Run `hob run <file.lua> [args...]`.
pub(crate) fn file(
    rest: &[String],
    control: Control,
    trace: Option<PathBuf>,
    err: &mut dyn Write,
) -> u8 {
    let Some(path) = rest.first() else {
        return fail(
            err,
            &format!("`run` needs a flow file\n\n{}", super::help::text()),
        );
    };
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => return report(err, &format!("cannot read `{path}`: {error}"), 1),
    };
    let name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("flow");
    execute(&source, name, &rest[1..], control, trace, err)
}

/// Run a command the registry resolved.
pub(crate) fn command(
    command: &Command,
    args: &[String],
    control: Control,
    trace: Option<PathBuf>,
    err: &mut dyn Write,
) -> u8 {
    let source = match fs::read_to_string(&command.path) {
        Ok(source) => source,
        Err(error) => {
            return report(
                err,
                &format!("cannot read `{}`: {error}", command.path.display()),
                1,
            );
        }
    };
    execute(&source, &command.name, args, control, trace, err)
}

/// Drive one flow and map its outcome to an exit code.
fn execute(
    source: &str,
    name: &str,
    args: &[String],
    control: Control,
    trace: Option<PathBuf>,
    err: &mut dyn Write,
) -> u8 {
    match driver::run(name, source, args, control, trace) {
        Ok(()) => 0,
        Err(failure) if failure.message.is_empty() => failure.code,
        Err(failure) => report(err, &failure.message, failure.code),
    }
}

/// Print a store error and return its exit code.
pub(crate) fn finish(result: Result<(), Failure>, err: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => 0,
        Err(failure) => report(err, &failure.message, failure.code),
    }
}

/// Print to `err` and return `USAGE_EXIT`.
pub(crate) fn fail(err: &mut dyn Write, message: &str) -> u8 {
    report(err, message, USAGE_EXIT)
}

/// Print `error: ...` and return the code.
pub(crate) fn report(err: &mut dyn Write, message: &str, code: u8) -> u8 {
    if !message.is_empty() {
        let _ = writeln!(err, "error: {message}");
    }
    code
}

/// Print to `out`, or 1 when the stream is gone.
pub(crate) fn write(out: &mut dyn Write, text: &str) -> u8 {
    u8::from(out.write_all(text.as_bytes()).is_err())
}
