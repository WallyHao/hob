// --- builtin::doctor ---
// `hob doctor`: what the installation looks like from the inside.
//
// It reports paths and counts, and whether each provider's key variable is set,
// by name only: a doctor that printed a value would be a leak with a friendly
// face. `--json` prints the same facts as one object.

use std::io::Write;

use crate::cli::report::Format;
use crate::effect::Failure;
use crate::paths::Paths;
use crate::store::{self, Args, Command, Meta, Origin, Source};

use super::report;

/// The builtin itself.
pub(crate) fn command() -> Command {
    Command {
        name: "doctor".to_owned(),
        origin: Origin::Builtin,
        source: Source::Builtin,
        meta: Meta {
            summary: Some("Report the configuration, the project and the keys.".to_owned()),
            usage: Some("doctor".to_owned()),
            args: Args::parse("0"),
        },
    }
}

/// Print the state of the installation.
// The builtin table stays uniform: every entry returns a Result so the next one
// may fail, and `builtin::run` has one shape to map.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn run(out: &mut dyn Write, format: Format) -> Result<(), Failure> {
    let paths = Paths::resolve();
    let listing = store::scan(&paths);
    match format {
        Format::Text => report::text(&paths, &listing, out),
        Format::Json => report::json(&paths, &listing, out),
    }
    Ok(())
}
