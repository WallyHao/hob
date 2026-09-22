// --- effect::ops::logs ---
// Arguments of `logs.write`.

use serde::Deserialize;

/// Severity of one log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Level {
    /// Fine-grained tracing, shown under `-vvv`.
    Trace,
    /// Developer-facing detail, shown under `-vv`.
    Debug,
    /// Progress worth reading.
    Info,
    /// Something recoverable went wrong.
    Warn,
    /// Something failed.
    Error,
}

impl Level {
    /// The word a log line is prefixed with.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    /// Severity as a number, compared against the verbosity flag.
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::Trace => 3,
            Self::Debug => 2,
            Self::Info => 1,
            Self::Warn | Self::Error => 0,
        }
    }
}

/// One log entry.
#[derive(Debug, Deserialize)]
pub(crate) struct Write {
    /// Severity.
    pub(crate) level: Level,
    /// Text of the line.
    pub(crate) msg: String,
}
