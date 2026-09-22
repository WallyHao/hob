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

/// `term.input`.
#[derive(Debug, Deserialize)]
pub(crate) struct Input {
    /// Question shown before reading.
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    /// Answer used when the line is empty.
    #[serde(default)]
    pub(crate) default: Option<String>,
    /// Accepted as a second default: hob has no line editor to pre-fill.
    #[serde(default)]
    pub(crate) initial: Option<String>,
}

/// `term.allow`.
#[derive(Debug, Deserialize)]
pub(crate) struct Allow {
    /// Question shown before reading.
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    /// Answer used when the line is empty; false when absent.
    #[serde(default)]
    pub(crate) default: Option<bool>,
    /// Text printed before the question, e.g. what is about to happen.
    #[serde(default)]
    pub(crate) detail: Option<String>,
}

/// One selectable option: a bare label, or a label with a value to return.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Choice {
    /// The label is the value.
    Label(String),
    /// The label is shown, the value is returned.
    Pair {
        /// Text shown in the list.
        label: String,
        /// Value returned when this option is picked.
        value: serde_json::Value,
    },
}

impl Choice {
    /// Text shown in the list.
    pub(crate) fn label(&self) -> &str {
        match self {
            Self::Label(label) | Self::Pair { label, .. } => label,
        }
    }

    /// Value returned when this option is picked.
    pub(crate) fn value(&self) -> serde_json::Value {
        match self {
            Self::Label(label) => serde_json::Value::String(label.clone()),
            Self::Pair { value, .. } => value.clone(),
        }
    }
}

/// `term.select`.
#[derive(Debug, Deserialize)]
pub(crate) struct Select {
    /// Question shown above the list.
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    /// Options to pick from.
    pub(crate) options: Vec<Choice>,
    /// Label picked when the line is empty.
    #[serde(default)]
    pub(crate) default: Option<String>,
}

/// `term.choose`.
#[derive(Debug, Deserialize)]
pub(crate) struct Choose {
    /// Question shown above the list.
    #[serde(default)]
    pub(crate) prompt: Option<String>,
    /// Options to pick from.
    pub(crate) options: Vec<Choice>,
    /// Labels picked when the line is empty.
    #[serde(default)]
    pub(crate) defaults: Vec<String>,
    /// Fewest picks accepted.
    #[serde(default)]
    pub(crate) min: Option<usize>,
    /// Most picks accepted.
    #[serde(default)]
    pub(crate) max: Option<usize>,
}
