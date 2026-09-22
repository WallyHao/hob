// --- store::scan ---
// Directory walking. A tier that does not exist is simply empty; a tier that
// cannot be read is reported, because silently losing every command in it
// would look like the commands were never written.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::paths::Paths;

use super::{Command, Ignored, Listing, Origin, RESERVED, valid_name};

/// Collect the registry, project layer first.
pub(crate) fn scan(paths: &Paths) -> Listing {
    let mut listing = Listing::default();
    let mut tiers: Vec<(Origin, PathBuf)> = Vec::new();
    if let Some(dir) = paths.project_commands() {
        tiers.push((Origin::Project, dir));
    }
    if let Some(dir) = paths.user_commands() {
        tiers.push((Origin::User, dir));
    }
    for (origin, dir) in tiers {
        collect(&dir, origin, &mut listing);
    }
    listing
}

/// Add every `.lua` file directly under one tier.
fn collect(dir: &Path, origin: Origin, listing: &mut Listing) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(_) => {
            listing.ignored.push(Ignored {
                path: dir.to_path_buf(),
                reason: "unreadable directory",
            });
            return;
        }
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some("lua") || !path.is_file() {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !valid_name(name) {
            listing.ignored.push(Ignored {
                path,
                reason: "name must match [a-z][a-z0-9-]*",
            });
            continue;
        }
        if RESERVED.contains(&name) {
            listing.ignored.push(Ignored {
                path,
                reason: "reserved for the CLI",
            });
            continue;
        }
        listing.commands.push(Command {
            name: name.to_owned(),
            origin,
            summary: summary(&path),
            path,
        });
    }
}

/// The first non-empty line, when it is a `---` doc comment.
fn summary(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        return line
            .strip_prefix("--- ")
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned);
    }
    None
}
