// --- effect::ops::term ---
// Arguments of `term.print`.

use serde::Deserialize;

/// Named styling, so a flow never spells an escape code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Style {
    /// Something failed.
    Error,
    /// Something recoverable went wrong.
    Warn,
    /// Neutral information.
    Info,
    /// Something worked.
    Success,
    /// Secondary text.
    Muted,
    /// A heading.
    Title,
}

impl Style {
    /// The ANSI SGR parameter for this style.
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Error => "31",
            Self::Warn => "33",
            Self::Info => "34",
            Self::Success => "32",
            Self::Muted => "2",
            Self::Title => "1",
        }
    }
}

/// Which stream a line goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Stream {
    /// Standard output, the default.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One line of output.
#[derive(Debug, Deserialize)]
pub(crate) struct Print {
    /// The text to print.
    pub(crate) text: String,
    /// Optional styling.
    #[serde(default)]
    pub(crate) style: Option<Style>,
    /// Optional stream override.
    #[serde(default)]
    pub(crate) stream: Option<Stream>,
}
