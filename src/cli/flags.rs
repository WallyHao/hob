// --- cli::flags ---
// The control flags: how much of a run to perform and how loud to be.
//
// They may appear before or after the command name, because `hob git-commit
// --dry-run` is how the flag is naturally typed. Everything after `--` belongs
// to the flow, so a command that has a flag of its own can still receive it.

use std::path::PathBuf;

use crate::driver::{Control, Mode};

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
        trace: Option<PathBuf>,
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
    let mut words = Vec::new();
    let mut rest = args.into_iter();
    while let Some(word) = rest.next() {
        let word = word.as_ref();
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
            "-y" | "--yes" => yes = true,
            "-q" | "--quiet" => quiet = true,
            "--verbose" => verbose += 1,
            _ if is_verbose(word) => verbose += u8::try_from(word.len() - 1).unwrap_or(u8::MAX),
            _ if word.starts_with('-') && words.is_empty() => {
                return Err(format!("unknown argument `{word}`"));
            }
            _ => words.push(word.to_owned()),
        }
    }
    if quiet && verbose > 0 {
        return Err("`-q` and `-v` cannot be combined".to_owned());
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
        },
        trace,
        words,
    })
}

/// Whether a word is a run of `v`s after one dash, e.g. `-vv`.
fn is_verbose(word: &str) -> bool {
    word.len() > 1 && word.starts_with('-') && word[1..].bytes().all(|byte| byte == b'v')
}

/// Combine a mode flag with the one already seen.
fn pick(current: Mode, wanted: Mode, flag: &str) -> Result<Mode, String> {
    if current == Mode::Run || current == wanted {
        return Ok(wanted);
    }
    let other = if current == Mode::DryRun {
        "--dry-run"
    } else {
        "--step"
    };
    Err(format!("`{flag}` conflicts with `{other}`"))
}
