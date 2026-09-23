// --- exec::term::color ---
// When a style becomes an escape code.
//
// Precedence: the `--color` flag first, then the environment, then the stream.
// `NO_COLOR` and `CLICOLOR_FORCE` follow the conventions of no-color.org and
// clicolor; a pipe gets plain text unless something asked for colour.

use std::ffi::OsStr;

/// How styled output is treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Color {
    /// Colour when the stream is a terminal.
    #[default]
    Auto,
    /// Colour even into a pipe.
    Always,
    /// Never colour.
    Never,
}

impl Color {
    /// Whether a style may be spelled on this stream.
    pub(crate) fn allows(self, terminal: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => {
                if non_empty("NO_COLOR") {
                    return false;
                }
                forced() || terminal
            }
        }
    }
}

/// Whether a variable is set to anything, including `0`.
fn non_empty(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

/// `CLICOLOR_FORCE`, where `0` means "do not force" rather than "set".
fn forced() -> bool {
    std::env::var_os("CLICOLOR_FORCE")
        .is_some_and(|value| !value.is_empty() && value.as_os_str() != OsStr::new("0"))
}
