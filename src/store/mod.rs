// --- store ---
// The command registry: `.hob/commands/*.lua` in the project first, then the
// user's `commands/`, nearest wins.
//
// Discovery never executes a command. A file contributes a name, a path and
// its first `---` line; the source is read only when the command runs, so
// `hob list` cannot be tricked into running anything.

mod change;
mod create;
mod query;
mod scan;
mod suggest;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::cli::USAGE_EXIT;
use crate::effect::Failure;

pub(crate) use change::remove;
pub(crate) use create::new;
pub(crate) use query::{list, unknown, which};
pub(crate) use scan::scan;

/// Names the CLI itself owns; a command may not shadow them.
pub(crate) const RESERVED: &[&str] = &["list", "new", "rm", "run", "which"];

/// Which layer a command came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    Project,
    User,
}

impl Origin {
    /// The word `list` and `which` print.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

/// One candidate command, in precedence order.
#[derive(Debug, Clone)]
pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) origin: Origin,
    pub(crate) path: PathBuf,
    pub(crate) summary: Option<String>,
}

/// A file that looks like a command but is not one.
#[derive(Debug)]
pub(crate) struct Ignored {
    pub(crate) path: PathBuf,
    pub(crate) reason: &'static str,
}

/// Everything discovery found.
#[derive(Debug, Default)]
pub(crate) struct Listing {
    /// Candidates in precedence order; a repeated name means a shadow.
    pub(crate) commands: Vec<Command>,
    pub(crate) ignored: Vec<Ignored>,
}

impl Listing {
    /// The command a bare name resolves to.
    pub(crate) fn effective(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.name == name)
    }

    /// Every candidate for a name, most specific first.
    pub(crate) fn chain(&self, name: &str) -> impl Iterator<Item = &Command> {
        self.commands
            .iter()
            .filter(move |command| command.name == name)
    }

    /// Effective commands, one per name, sorted for display.
    pub(crate) fn effective_all(&self) -> Vec<&Command> {
        let mut by_name = BTreeMap::new();
        for command in &self.commands {
            by_name.entry(command.name.as_str()).or_insert(command);
        }
        by_name.into_values().collect()
    }
}

/// Whether a file stem may name a command.
pub(crate) fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    name.len() <= 32 && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Exactly one name, for the verbs that take nothing else.
pub(crate) fn one<'a>(verb: &str, rest: &'a [String]) -> Result<&'a str, Failure> {
    match rest {
        [name] => Ok(name),
        [] => Err(Failure::with_code(
            format!("`{verb}` needs a name"),
            USAGE_EXIT,
        )),
        _ => Err(Failure::with_code(
            format!("`{verb}` takes one name"),
            USAGE_EXIT,
        )),
    }
}
