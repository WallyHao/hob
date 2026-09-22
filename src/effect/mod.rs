// --- effect ---
// The wire format Lua uses to ask the engine for work.
//
// A flow never performs I/O. It yields one effect and blocks until the engine
// resumes it with the result:
//
//     { ns = "term", op = "print", cmd = { text = "hi" } }
//
// `ns` selects the subsystem, `op` the operation inside it, and `cmd` carries
// the arguments. `try = true` asks for `(nil, message)` instead of an abort.
// `safety` is already on the wire; the preview that reads it arrives with
// `--dry-run`.

use serde_json::{Map, Value};

pub(crate) mod bridge;
pub(crate) mod ops;

pub(crate) use bridge::{json_to_lua, lua_to_json};

/// Field carrying an abort message back into the coroutine.
///
/// Must match `ABORT_KEY` in `lua/hob/init.lua`; an integration test fails if
/// the two ever drift apart.
pub(crate) const ABORT_KEY: &str = "__hob_abort";

/// Why an effect did not produce a result, and the exit code that follows.
#[derive(Debug, Clone)]
pub(crate) struct Failure {
    /// Message shown to the user.
    pub(crate) message: String,
    /// Exit code the flow ends with.
    pub(crate) code: u8,
}

impl Failure {
    /// A failure that exits with code 1.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
        }
    }

    /// A failure with an explicit exit code.
    pub(crate) fn with_code(message: impl Into<String>, code: u8) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }

    /// No effect is registered under that `ns.op`.
    pub(crate) fn unknown(ns: &str, op: &str) -> Self {
        Self::new(format!("unknown effect `{ns}.{op}`"))
    }
}

/// Value the driver resumes with when a non-fallible effect failed.
///
/// A failure has to be delivered as data because only code running inside the
/// coroutine may raise: resuming with this table lets `hob.effect` turn it
/// into a Lua error, which the flow can still catch with `pcall`.
pub(crate) fn abort(message: &str) -> Value {
    let mut map = Map::new();
    map.insert(ABORT_KEY.to_owned(), Value::String(message.to_owned()));
    Value::Object(map)
}

impl From<mlua::Error> for Failure {
    fn from(error: mlua::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// One decoded effect request.
#[derive(Debug, Clone)]
pub(crate) struct Request {
    /// Subsystem that owns the operation, e.g. `term`.
    pub(crate) ns: String,
    /// Operation inside the subsystem, e.g. `print`.
    pub(crate) op: String,
    /// Argument object; empty when the operation takes none.
    pub(crate) cmd: Value,
    /// Resume with `(nil, message)` rather than aborting on failure.
    pub(crate) fallible: bool,
}

impl Request {
    /// Read a yielded Lua value as a request.
    ///
    /// Rejects anything that is not a table with string `ns` and `op`, because
    /// a malformed request means the stdlib is broken and guessing would hide
    /// the bug.
    pub(crate) fn from_lua(value: &mlua::Value) -> Result<Self, Failure> {
        let Value::Object(mut fields) =
            lua_to_json(value).map_err(|error| Failure::new(error.to_string()))?
        else {
            return Err(Failure::new(format!(
                "effect must be a table, got {}",
                value.type_name()
            )));
        };
        let ns = string_field(&fields, "ns")?;
        let op = string_field(&fields, "op")?;
        // An empty Lua table encodes as `null`, so a missing or empty `cmd`
        // both mean "no arguments".
        let cmd = match fields.remove("cmd").unwrap_or(Value::Null) {
            Value::Object(arguments) => Value::Object(arguments),
            Value::Null => Value::Object(Map::new()),
            other => {
                return Err(Failure::new(format!("`cmd` must be a table, got {other}")));
            }
        };
        Ok(Self {
            ns,
            op,
            cmd,
            fallible: fields.get("try").and_then(Value::as_bool).unwrap_or(false),
        })
    }
}

fn string_field(fields: &Map<String, Value>, key: &str) -> Result<String, Failure> {
    fields
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Failure::new(format!("effect is missing `{key}`")))
}
