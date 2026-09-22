// --- cli::help ---

use super::{NAME, VERSION};

/// The help text, shown for `--help` and every usage error.
pub(crate) fn text() -> String {
    format!(
        r"{NAME} {VERSION}
Scriptable CLI for AI chores.

Usage: {NAME} <command> [options]

Commands:
  run <file.lua> [args...]     Run a flow file
  list                         List installed commands
  new <name> [--user|--local]  Create a command
  rm <name>                    Remove a command
  which <name>                 Show where a command comes from
  <name> [args...]             Run an installed command

Options:
  -h, --help     Print help
  -V, --version  Print version
"
    )
}
