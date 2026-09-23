// --- cli::flags::values ---
// Small value parsers and combiners, kept beside the parser rather than in it.

use std::time::Duration;

use crate::driver::Mode;
use crate::exec::term::Color;

/// The value of a flag that takes one, either as the next word or after `=`.
pub(super) fn value<S: AsRef<str>>(
    rest: &mut impl Iterator<Item = S>,
    flag: &str,
) -> Result<S, String> {
    rest.next().ok_or_else(|| format!("`{flag}` needs a value"))
}

/// A whole number for a cap flag; zero means no cap.
pub(super) fn count(text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| format!("`{text}` is not a whole number"))
}

/// The highest verbosity: `-vv` already shows trace lines, so a longer run of
/// `v`s is accepted and means the same rather than overflowing.
pub(super) const MAX_VERBOSITY: u8 = 2;

/// Whether a word is a run of `v`s after one dash, e.g. `-vv`.
pub(super) fn is_verbose(word: &str) -> bool {
    word.len() > 1 && word.starts_with('-') && word[1..].bytes().all(|byte| byte == b'v')
}

/// Combine a mode flag with the one already seen.
pub(super) fn pick(current: Mode, wanted: Mode, flag: &str) -> Result<Mode, String> {
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

/// How styled output should be treated.
pub(super) fn color_of(text: &str) -> Result<Color, String> {
    match text {
        "auto" => Ok(Color::Auto),
        "always" => Ok(Color::Always),
        "never" => Ok(Color::Never),
        other => Err(format!(
            "`{other}` is not a colour mode; use auto, always or never"
        )),
    }
}

/// Whole seconds, at least one.
pub(super) fn seconds(text: &str) -> Result<Duration, String> {
    let value = text
        .parse::<u64>()
        .map_err(|_| format!("`{text}` is not a number of seconds"))?;
    if value == 0 {
        return Err("`--timeout` must be at least one second".to_owned());
    }
    Ok(Duration::from_secs(value))
}
