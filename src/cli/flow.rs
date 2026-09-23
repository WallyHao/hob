// --- cli::flow ---
// Running a flow, whether it came from a path (`hob run`) or from the
// registry (a bare command name), plus the reporting shared by every verb.

use std::fs;
use std::io::Write;
use std::path::Path;

use super::report::{self, Format};
use crate::driver::{self, Control};
use crate::store::trust::denial;
use crate::store::{Command, Source};
use crate::trace::Settings;

use super::USAGE_EXIT;

/// Run `hob run <file.lua> [args...]`.
pub(crate) fn file(
    rest: &[String],
    control: Control,
    trace: Option<Settings>,
    format: Format,
    err: &mut dyn Write,
) -> u8 {
    let Some(path) = rest.first() else {
        return report::error(
            err,
            &format!("`run` needs a flow file\n\n{}", super::help::text()),
            USAGE_EXIT,
            format,
        );
    };
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return report::error(err, &format!("cannot read `{path}`: {error}"), 1, format);
        }
    };
    let name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("flow");
    execute(&source, name, &rest[1..], control, trace, format, err)
}

/// Run a command the registry resolved.
pub(crate) fn command(
    command: &Command,
    args: &[String],
    control: Control,
    trace: Option<Settings>,
    format: super::report::Format,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    // `--help` is the command's own only when it is the whole argument list, so
    // a flow that takes a `--help` of its own still receives it.
    if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help") {
        return describe(command, out);
    }
    if let Some(spec) = command.meta.args
        && let Err(message) = spec.check(args.len(), &usage(command))
    {
        return report::error(err, &message, USAGE_EXIT, format);
    }
    // A project command is code; running it needs the checkout to be trusted.
    if let Some(failure) = denial(command) {
        return report::error(err, &failure.message, failure.code, format);
    }
    let Source::File(path) = &command.source else {
        return report::finish(crate::builtin::run(&command.name, out, format), err, format);
    };
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return report::error(
                err,
                &format!("cannot read `{}`: {error}", path.display()),
                1,
                format,
            );
        }
    };
    execute(&source, &command.name, args, control, trace, format, err)
}

/// Print what a command says about itself, without running it.
fn describe(command: &Command, out: &mut dyn Write) -> u8 {
    let summary = command.meta.summary.as_deref().unwrap_or("no summary");
    let _ = writeln!(out, "{} -- {summary}", command.name);
    let _ = writeln!(out, "usage: {}", usage(command));
    if let Some(args) = command.meta.args {
        let _ = writeln!(out, "args:  {}", args.describe());
    }
    let from = command.path().map_or_else(
        || "compiled in".to_owned(),
        |path| format!("{} ({})", path.display(), command.origin.label()),
    );
    let _ = writeln!(out, "from:  {from}");
    0
}

/// The usage line: the header's, or a derived default.
fn usage(command: &Command) -> String {
    command
        .meta
        .usage
        .clone()
        .unwrap_or_else(|| format!("{} [args...]", command.name))
}

/// Drive one flow and map its outcome to an exit code.
fn execute(
    source: &str,
    name: &str,
    args: &[String],
    control: Control,
    trace: Option<Settings>,
    format: Format,
    err: &mut dyn Write,
) -> u8 {
    match driver::run(name, source, args, control, trace) {
        Ok(()) => 0,
        Err(failure) if failure.message.is_empty() => failure.code,
        Err(failure) => report::error(err, &failure.message, failure.code, format),
    }
}
