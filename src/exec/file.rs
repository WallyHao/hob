// --- exec::file ---
// File access, the only way a flow can reach a file. Every call is an effect,
// so a preview can see it before it happens and nothing here needs to guess
// whether an operation was safe.

mod atomic;

use std::fs;
use std::io::{ErrorKind, Read as _, Write as _};
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::file::{List, Read, Stat, Write};

/// The most a read brings in when the flow names no limit.
pub(crate) const READ_LIMIT: u64 = 8 * 1024 * 1024;

/// Read a UTF-8 file, at most `limit` bytes of it.
pub(crate) fn read(request: &Read) -> Result<Value, Failure> {
    let limit = request.limit.unwrap_or(READ_LIMIT);
    let mut file = match fs::File::open(&request.path) {
        Ok(file) => file,
        Err(error) if request.optional && error.kind() == ErrorKind::NotFound => {
            return Ok(Value::Null);
        }
        Err(error) => {
            return Err(Failure::new(format!(
                "cannot read `{}`: {error}",
                request.path
            )));
        }
    };
    let mut text = String::new();
    // One byte past the cap tells "exactly at it" from "over it".
    let read = if limit == 0 {
        file.read_to_string(&mut text)
    } else {
        file.take(limit + 1).read_to_string(&mut text)
    };
    read.map_err(|error| Failure::new(format!("cannot read `{}`: {error}", request.path)))?;
    if limit > 0 && text.len() as u64 > limit {
        return Err(Failure::new(format!(
            "`{}` is bigger than the {limit}-byte read limit; pass `limit = 0` to read it anyway",
            request.path
        )));
    }
    Ok(Value::String(text))
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
    if request.append {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&request.path)
            .and_then(|mut file| file.write_all(request.text.as_bytes()))
            .map_err(|error| Failure::new(format!("cannot write `{}`: {error}", request.path)))?;
        return Ok(Value::Null);
    }
    atomic::replace(Path::new(&request.path), &request.text)?;
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
