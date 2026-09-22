// --- exec::file ---
// File access, the only way a flow can reach a file. Every call is an effect,
// so a preview can see it before it happens and nothing here needs to guess
// whether an operation was safe.

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::file::{List, Read, Stat, Write};

/// Read a UTF-8 file.
pub(crate) fn read(request: &Read) -> Result<Value, Failure> {
    match fs::read_to_string(&request.path) {
        Ok(text) => Ok(Value::String(text)),
        Err(error) if request.optional && error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Value::Null)
        }
        Err(error) => Err(Failure::new(format!(
            "cannot read `{}`: {error}",
            request.path
        ))),
    }
}

/// Write or append text, creating parent directories.
pub(crate) fn write(request: &Write) -> Result<Value, Failure> {
    if let Some(parent) = Path::new(&request.path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| {
            Failure::new(format!("cannot create `{}`: {error}", parent.display()))
        })?;
    }
    let result = if request.append {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&request.path)
            .and_then(|mut file| file.write_all(request.text.as_bytes()))
    } else {
        fs::write(&request.path, &request.text)
    };
    result.map_err(|error| Failure::new(format!("cannot write `{}`: {error}", request.path)))?;
    Ok(Value::Null)
}

/// Size, mtime and kind of a path, or `nil` when it does not exist.
pub(crate) fn stat(request: &Stat) -> Result<Value, Failure> {
    let metadata = match fs::symlink_metadata(&request.path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Value::Null),
        Err(error) => {
            return Err(Failure::new(format!(
                "cannot stat `{}`: {error}",
                request.path
            )));
        }
    };
    let file_type = metadata.file_type();
    let kind = if file_type.is_symlink() {
        "link"
    } else if file_type.is_dir() {
        "dir"
    } else {
        "file"
    };
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_secs());
    Ok(serde_json::json!({
        "size": metadata.len(),
        "mtime": mtime,
        "kind": kind,
    }))
}

/// Entry names directly under a directory, sorted.
pub(crate) fn list(request: &List) -> Result<Value, Failure> {
    let entries = fs::read_dir(&request.path)
        .map_err(|error| Failure::new(format!("cannot list `{}`: {error}", request.path)))?;
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(Value::Array(names.into_iter().map(Value::String).collect()))
}
