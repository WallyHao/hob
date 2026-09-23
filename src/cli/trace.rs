// --- cli::trace ---
// A JSONL record of every effect a run requested and settled.
//
// The file is created fresh per run: a trace describes one run, and appending
// would mix runs without a boundary between them. A write failure never aborts
// the flow -- the trace is a diagnostic, and a full disk should not be the
// reason a flow fails -- so the first failure is reported and the trace is
// abandoned.
//
// Arguments are recorded, but values the environment calls secret are masked
// first, so a key cannot end up at rest in the trace.

use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::Value;

use crate::effect::{Failure, Request};
use crate::exec::redact::Redactor;

/// Where and how a trace is written.
#[derive(Debug, Clone)]
pub(crate) struct Settings {
    /// File the trace is written to.
    pub(crate) path: PathBuf,
    /// Record arguments as they are, rather than as their size.
    pub(crate) full: bool,
}

/// Writes one JSON object per line as effects are requested and settled.
#[derive(Debug)]
pub(crate) struct Trace {
    file: Option<BufWriter<std::fs::File>>,
    redact: Redactor,
    full: bool,
    started: Instant,
}

impl Trace {
    /// Create the trace file, replacing whatever was there.
    pub(crate) fn create(path: &Path, full: bool) -> Result<Self, Failure> {
        let file = std::fs::File::create(path)
            .map_err(|error| Failure::new(format!("cannot write `{}`: {error}", path.display())))?;
        Ok(Self {
            file: Some(BufWriter::new(file)),
            redact: Redactor::from_env(),
            full,
            started: Instant::now(),
        })
    }

    /// Record that the flow asked for an effect.
    pub(crate) fn request(&mut self, request: &Request) {
        let mut cmd = self.redact.value(&request.cmd);
        if !self.full {
            summarize(&request.ns, &request.op, &mut cmd);
        }
        self.write(serde_json::json!({
            "event": "request",
            "ns": request.ns,
            "op": request.op,
            "cmd": cmd,
            "fallible": request.fallible,
        }));
    }

    /// Record how an effect ended.
    ///
    /// Only the identity is repeated, not the arguments: the request line
    /// already carries them, and repeating them would double the file.
    pub(crate) fn outcome(&mut self, request: &Request, outcome: &str, error: Option<&str>) {
        self.write(serde_json::json!({
            "event": "outcome",
            "ns": request.ns,
            "op": request.op,
            "outcome": outcome,
            "error": error,
        }));
    }

    /// Write one line, giving up on the trace rather than the flow on failure.
    fn write(&mut self, mut record: serde_json::Value) {
        let Some(file) = self.file.as_mut() else {
            return;
        };
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if let Some(fields) = record.as_object_mut() {
            fields.insert("t".to_owned(), serde_json::Value::from(elapsed));
        }
        let result = serde_json::to_writer(&mut *file, &record)
            .map_err(|error| error.to_string())
            .and_then(|()| file.write_all(b"\n").map_err(|error| error.to_string()))
            .and_then(|()| file.flush().map_err(|error| error.to_string()));
        if let Err(error) = result {
            let _ = writeln!(io::stderr(), "warn: trace disabled: {error}");
            self.file = None;
        }
    }
}

/// Replace the fields that carry user content with their size, so a trace is
/// safe to keep or share unless `--trace-full` said otherwise.
fn summarize(ns: &str, op: &str, cmd: &mut Value) {
    match (ns, op) {
        ("file", "write") => mask_text(cmd, "text"),
        ("agent", "ask" | "send") => {
            mask_text(cmd, "prompt");
            mask_text(cmd, "system");
            mask_messages(cmd);
        }
        ("agent", "open") => mask_text(cmd, "system"),
        _ => {}
    }
}

/// One string field becomes its size.
fn mask_text(cmd: &mut Value, key: &str) {
    let bytes = match cmd.get(key) {
        Some(Value::String(text)) => text.len(),
        _ => return,
    };
    cmd[key] = Value::String(format!("<{bytes} bytes>"));
}

/// The message list becomes its length.
fn mask_messages(cmd: &mut Value) {
    if matches!(cmd.get("messages"), Some(Value::Array(_))) {
        let count = cmd["messages"].as_array().map_or(0, Vec::len);
        cmd["messages"] = Value::String(format!("<{count} messages>"));
    }
}
