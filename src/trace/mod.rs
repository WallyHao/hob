// --- trace ---
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

use crate::effect::{Failure, Operation, Request};
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
    sequence: u64,
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
            sequence: 0,
        })
    }

    /// Record that the flow asked for an effect.
    pub(crate) fn request(&mut self, request: &Request) {
        self.sequence += 1;
        let mut cmd = self.redact.value(&request.cmd);
        if !self.full {
            summarize(request, &mut cmd);
        }
        self.write(&serde_json::json!({
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
        self.write(&serde_json::json!({
            "event": "outcome",
            "ns": request.ns,
            "op": request.op,
            "outcome": outcome,
            "error": error,
        }));
    }

    /// Write one line, giving up on the trace rather than the flow on failure.
    fn write(&mut self, record: &serde_json::Value) {
        let mut record = self.redact.value(record);
        let Some(file) = self.file.as_mut() else {
            return;
        };
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if let Some(fields) = record.as_object_mut() {
            fields.insert("id".to_owned(), Value::from(self.sequence));
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

// --- content summaries ---
// Invalid requests use the union of content fields rather than leaking a bad input.
fn summarize(request: &Request, cmd: &mut Value) {
    let operation = Operation::parse(request);
    let fields = operation.as_ref().map_or(
        &[
            "text", "prompt", "system", "messages", "message", "stdin", "line", "profile", "env",
            "value", "msg",
        ][..],
        |operation| operation.policy().1,
    );
    for key in fields {
        if let Some(value) = cmd.get_mut(*key) {
            let summary = match &*value {
                Value::String(text) => format!("<{} bytes>", text.len()),
                Value::Array(items) => format!("<{} items>", items.len()),
                other => format!("<{} bytes>", other.to_string().len()),
            };
            *value = Value::String(summary);
        }
    }
}
