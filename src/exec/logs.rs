// --- exec::logs ---
// Log lines go to stderr so stdout stays data a caller can pipe. A level is
// shown when it is at or below the run's verbosity: warnings and errors always,
// `info` by default, `debug` under `-vv`, `trace` under `-vvv`.

use std::io::Write as _;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::logs::Write;
use crate::exec::RunContext;

/// Write one log line.
// Every operation returns a `Result` so the dispatch table stays uniform;
// logging cannot fail, but the next effect in this namespace may.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn write(state: &RunContext, entry: &Write) -> Result<Value, Failure> {
    if entry.level.rank() <= state.verbosity {
        // A diagnostic is not worth panicking over a closed stderr.
        let _ = writeln!(std::io::stderr(), "{}: {}", entry.level.label(), entry.msg);
    }
    Ok(Value::Null)
}
