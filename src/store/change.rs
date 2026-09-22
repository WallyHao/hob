// --- store::change ---
// The writing verb: rm. It only ever deletes a file the registry resolved,
// never a path typed by the user, so a typo cannot reach anything else.

use std::fs;
use std::io::Write;

use crate::effect::Failure;

use super::{Listing, one};

/// Delete the effective file for a name.
pub(crate) fn remove(
    listing: &Listing,
    rest: &[String],
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let name = one("rm", rest)?;
    let command = listing
        .effective(name)
        .ok_or_else(|| Failure::new(super::unknown(name, listing)))?;
    let Some(path) = command.path() else {
        return Err(Failure::new(format!(
            "`{name}` is a builtin and cannot be removed"
        )));
    };
    fs::remove_file(path)
        .map_err(|error| Failure::new(format!("cannot remove {}: {error}", path.display())))?;
    let _ = writeln!(out, "removed {}", path.display());
    if let Some(next) = listing.chain(name).nth(1) {
        let _ = writeln!(
            out,
            "note: `{name}` now resolves to {}",
            next.path()
                .map_or_else(|| "a builtin".to_owned(), |path| path.display().to_string())
        );
    }
    Ok(())
}
