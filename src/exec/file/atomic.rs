// --- exec::file::atomic ---
// Replacing a file without ever leaving a half-written one: write a temp beside
// the target, then rename it over. A symlink is replaced where it points, the
// way a plain write through it would, and the target's mode is kept.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process;

use crate::effect::Failure;

/// Replace `path` with `text`, atomically.
pub(super) fn replace(path: &Path, text: &str) -> Result<(), Failure> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let temp = temp_path(&target);
    let result = write_and_rename(&temp, &target, text);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// A hidden name beside the target, so the rename stays on one filesystem.
fn temp_path(target: &Path) -> PathBuf {
    let name = target.file_name().map_or_else(
        || "file".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let directory = target.parent().unwrap_or_else(|| Path::new("."));
    directory.join(format!(".{name}.hob-tmp-{}", process::id()))
}

fn write_and_rename(temp: &Path, target: &Path, text: &str) -> Result<(), Failure> {
    let mut file = fs::File::create(temp)
        .map_err(|error| Failure::new(format!("cannot write `{}`: {error}", temp.display())))?;
    file.write_all(text.as_bytes())
        .map_err(|error| Failure::new(format!("cannot write `{}`: {error}", temp.display())))?;
    if let Ok(metadata) = fs::metadata(target) {
        let _ = fs::set_permissions(temp, metadata.permissions());
    }
    drop(file);
    fs::rename(temp, target)
        .map_err(|error| Failure::new(format!("cannot replace `{}`: {error}", target.display())))
}
