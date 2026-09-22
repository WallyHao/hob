// --- exec::proc::spawn ---
// Waiting for a child and collecting what it produced: the parts that have to
// be right for a timeout, a chatty command or a closed pipe not to hang the
// flow.

use std::path::Path;
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::effect::Failure;

/// How much of each stream is kept; the rest is counted, not stored.
pub(super) const LIMIT: usize = 1024 * 1024;

/// The exit code a timed-out command reports, as `timeout(1)` does.
pub(super) const TIMEOUT_CODE: i32 = 124;

/// Resolve a program on `PATH`, or `nil`.
// Every operation returns a `Result` so the dispatch table stays uniform;
// resolving cannot fail, but the next effect in this namespace may.
#[allow(clippy::unnecessary_wraps)]
pub(super) fn which(prog: &str) -> Result<Value, Failure> {
    let found = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(prog))
            .find(|candidate| executable(candidate))
    });
    Ok(found.map_or(Value::Null, |path| {
        Value::String(path.to_string_lossy().into_owned())
    }))
}

/// How the child ended.
pub(super) enum Waited {
    /// It exited on its own.
    Exited(ExitStatus),
    /// It was killed for running past the timeout.
    TimedOut,
}

/// Wait for the child, killing it when the timeout passes.
pub(super) fn wait(child: &mut Child, timeout_ms: Option<u64>) -> Result<Waited, Failure> {
    let Some(millis) = timeout_ms else {
        return child
            .wait()
            .map(Waited::Exited)
            .map_err(|error| Failure::new(format!("cannot wait for the child: {error}")));
    };
    let deadline = Instant::now() + Duration::from_millis(millis);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(Waited::Exited(status)),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(Waited::TimedOut);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                return Err(Failure::new(format!("cannot wait for the child: {error}")));
            }
        }
    }
}

/// Read a stream to the end, keeping at most `LIMIT` bytes and reporting
/// whether anything was dropped.
pub(super) fn drain(mut pipe: impl std::io::Read) -> (String, bool) {
    let mut kept = Vec::new();
    let mut total = 0;
    let mut buffer = [0u8; 8192];
    loop {
        match pipe.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                total += read;
                if kept.len() < LIMIT {
                    let room = LIMIT - kept.len();
                    kept.extend_from_slice(&buffer[..read.min(room)]);
                }
            }
        }
    }
    (
        String::from_utf8_lossy(&kept).into_owned(),
        total > kept.len(),
    )
}

pub(super) fn trim(text: String, trim: bool) -> String {
    if trim {
        text.trim_end().to_owned()
    } else {
        text
    }
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(path: &Path) -> bool {
    path.is_file()
}
