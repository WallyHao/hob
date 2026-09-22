// --- store::scan ---
// Directory walking. A tier that does not exist is simply empty; a tier that
// cannot be read is reported, because silently losing every command in it
// would look like the commands were never written.
//
// A subdirectory is a namespace: `foo/bar.lua` is the command `foo/bar`. A
// symlinked directory is not searched, so discovery cannot loop, but a
// symlinked file is a command like any other.

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
        collect(&dir, "", origin, &mut listing);
    }
    listing
}

/// Add every `.lua` file under one tier, depth first.
fn collect(dir: &Path, prefix: &str, origin: Origin, listing: &mut Listing) {
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
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let Some(segment) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if kind.is_dir() {
            let name = join(prefix, &segment);
            if valid_name(&name) {
                collect(&path, &name, origin, listing);
            } else {
                listing.ignored.push(Ignored {
                    path,
                    reason: "name must match [a-z][a-z0-9-]* segments",
                });
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("lua") {
            continue;
        }
        if !(kind.is_file() || kind.is_symlink() && path.is_file()) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let name = join(prefix, stem);
        if !valid_name(&name) {
            listing.ignored.push(Ignored {
                path,
                reason: "name must match [a-z][a-z0-9-]*",
            });
            continue;
        }
        if prefix.is_empty() && RESERVED.contains(&name.as_str()) {
            listing.ignored.push(Ignored {
                path,
                reason: "reserved for the CLI",
            });
            continue;
        }
        listing.commands.push(Command {
            name,
            origin,
            summary: summary(&path),
            path,
        });
    }
}

/// Join a namespace prefix and a segment.
fn join(prefix: &str, segment: &str) -> String {
    if prefix.is_empty() {
        segment.to_owned()
    } else {
        format!("{prefix}/{segment}")
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
