// --- builtin ---
// Commands compiled into the binary.
//
// A builtin is a command like any other from the outside: it has a name, a
// summary and a usage line, and a file in either layer shadows it. The layer
// exists for what a flow cannot report -- the state of the installation itself
// -- and is where a future `models` or `config` verb would land.

mod doctor;

use std::io::Write;

use crate::effect::Failure;
use crate::store::Command;

/// Every builtin, in precedence order after the file layers.
pub(crate) fn commands() -> Vec<Command> {
    vec![doctor::command()]
}

/// Run a builtin by name.
pub(crate) fn run(name: &str, out: &mut dyn Write) -> Result<(), Failure> {
    match name {
        "doctor" => doctor::run(out),
        other => Err(Failure::new(format!("unknown builtin `{other}`"))),
    }
}
