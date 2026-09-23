// --- cli::flags ---
// The control flags: how much of a run to perform and how loud to be.
//
// They may appear before or after the command name, because `hob git-commit
// --dry-run` is how the flag is naturally typed. Everything after `--` belongs
// to the flow, so a command that has a flag of its own can still receive it.

use std::path::PathBuf;

use super::report::Format;
use super::trace::Settings;
use crate::driver::{Control, Mode};

use controls::Controls;
use values::{MAX_VERBOSITY, is_verbose, pick};

mod controls;
mod values;

/// One parsed command line.
#[derive(Debug)]
pub(crate) enum Invocation {
    /// `--help`, or nothing at all.
    Help,
    /// `--version`.
    Version,
    /// A command, its control flags and its own words.
    Command {
        control: Control,
        trace: Option<Settings>,
        format: Format,
        words: Vec<String>,
    },
}

/// Split control flags from the words that name and feed the command.
pub(crate) fn parse<I, S>(args: I) -> Result<Invocation, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut mode = Mode::Run;
    let mut verbose = 0u8;
    let mut quiet = false;
    let mut yes = false;
    let mut help = false;
    let mut version = false;
    let mut trace: Option<PathBuf> = None;
    let mut trace_full = false;
    let mut format = Format::Text;
    let mut controls = Controls::default();
    let mut words = Vec::new();
    let mut rest = args.into_iter();
    while let Some(word) = rest.next() {
        let word = word.as_ref();
        if controls.read(word, &mut rest)? {
            continue;
        }
        match word {
            "--" => {
                words.extend(rest.map(|word| word.as_ref().to_owned()));
                break;
            }
            "-h" | "--help" if words.is_empty() => help = true,
            "-V" | "--version" if words.is_empty() => version = true,
            "--dry-run" => mode = pick(mode, Mode::DryRun, word)?,
            "--step" => mode = pick(mode, Mode::Step, word)?,
            "--trace" => {
                let path = rest
                    .next()
                    .ok_or_else(|| "`--trace` needs a file".to_owned())?;
                trace = Some(PathBuf::from(path.as_ref()));
            }
            _ if word.starts_with("--trace=") => {
                let path = word.trim_start_matches("--trace=");
                if path.is_empty() {
                    return Err("`--trace` needs a file".to_owned());
                }
                trace = Some(PathBuf::from(path));
            }
            "--trace-full" => trace_full = true,
            "--json" => format = Format::Json,
            "-y" | "--yes" => yes = true,
            "-q" | "--quiet" => quiet = true,
            "--verbose" => verbose = (verbose + 1).min(MAX_VERBOSITY),
            _ if is_verbose(word) => {
                verbose = verbose
                    .saturating_add(u8::try_from(word.len() - 1).unwrap_or(u8::MAX))
                    .min(MAX_VERBOSITY);
            }
            _ if word.starts_with('-') && words.is_empty() => {
                return Err(format!("unknown argument `{word}`"));
            }
            _ => words.push(word.to_owned()),
        }
    }
    if quiet && verbose > 0 {
        return Err("`-q` and `-v` cannot be combined".to_owned());
    }
    if trace_full && trace.is_none() {
        return Err("`--trace-full` needs `--trace`".to_owned());
    }
    if help || (words.is_empty() && !version) {
        return Ok(Invocation::Help);
    }
    if version {
        return Ok(Invocation::Version);
    }
    Ok(Invocation::Command {
        control: Control {
            mode,
            verbosity: if quiet { 0 } else { 1 + verbose },
            yes,
            timeout: controls.timeout,
            limits: controls.limits,
            color: controls.color,
        },
        trace: trace.map(|path| Settings {
            path,
            full: trace_full,
        }),
        format,
        words,
    })
}
