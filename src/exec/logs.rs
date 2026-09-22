// --- exec::logs ---
// Log lines go to stderr so stdout stays data a caller can pipe.

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::logs::Write;

/// Write one log line.
// Every operation returns a `Result` so the dispatch table stays uniform;
// logging cannot fail, but the next effect in this namespace may.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn write(entry: &Write) -> Result<Value, Failure> {
    eprintln!("{}: {}", entry.level.label(), entry.msg);
    Ok(Value::Null)
}
