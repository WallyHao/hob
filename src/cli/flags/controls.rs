// --- cli::flags::controls ---
// The flags that take a value: `--timeout`, `--max-calls`, `--max-tokens` and
// `--color`. They are read in one place so the main loop stays about words, and
// each accepts both `--flag value` and `--flag=value`.

use std::time::Duration;

use super::values::{color_of, count, seconds};
use crate::exec::budget::Limits;
use crate::exec::term::Color;

/// The control values one command line set.
#[derive(Debug, Default)]
pub(super) struct Controls {
    /// `--timeout`, in seconds.
    pub(super) timeout: Option<Duration>,
    /// `--max-calls` and `--max-tokens`.
    pub(super) limits: Limits,
    /// `--color`.
    pub(super) color: Color,
}

impl Controls {
    /// Read one of the value flags; `false` when the word is a different flag.
    pub(super) fn read<S: AsRef<str>>(
        &mut self,
        word: &str,
        rest: &mut impl Iterator<Item = S>,
    ) -> Result<bool, String> {
        let (flag, inline) = match word.split_once('=') {
            Some((flag, value)) => (flag, Some(value.to_owned())),
            None => (word, None),
        };
        if !matches!(
            flag,
            "--timeout" | "--max-calls" | "--max-tokens" | "--color"
        ) {
            return Ok(false);
        }
        let text = match inline {
            Some(value) => value,
            None => super::values::value(rest, flag)?.as_ref().to_owned(),
        };
        match flag {
            "--timeout" => self.timeout = Some(seconds(&text)?),
            "--max-calls" => self.limits.calls = count(&text)?,
            "--max-tokens" => self.limits.tokens = count(&text)?,
            _ => self.color = color_of(&text)?,
        }
        Ok(true)
    }
}
