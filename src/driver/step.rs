// --- driver::step ---
// One yielded effect: record it, ask the gate what to do, and turn the outcome
// into the value the coroutine resumes with.

use mlua::{Lua, MultiValue, Value};

use crate::effect::{Failure, Operation, Request, abort, json_to_lua};
use crate::exec;
use crate::trace::Trace;

use super::gate::{self, Gate};

/// Handle one yielded effect.
pub(super) fn handle(
    lua: &Lua,
    state: &mut exec::RunContext,
    gate: &mut Gate,
    trace: &mut Option<Trace>,
    yielded: &MultiValue,
) -> Result<MultiValue, Failure> {
    let first = yielded
        .front()
        .ok_or_else(|| Failure::new("the flow yielded no value"))?;
    let request = Request::from_lua(first)?;
    // `abort` is control flow, not work: the flow is over the moment it is
    // yielded, and nothing after it in the script ever runs.
    if request.ns == "term" && request.op == "abort" {
        return Err(abort_failure(&request));
    }
    if let Some(trace) = trace.as_mut() {
        trace.request(&request);
    }
    let operation = match Operation::parse(&request) {
        Ok(operation) => operation,
        Err(failure) => return outcome(lua, trace, &request, Err(failure)),
    };
    match gate.decide(state, &request, &operation)? {
        gate::Decision::Run => outcome(lua, trace, &request, exec::perform(state, &operation)),
        gate::Decision::Skip { reason, value } => {
            settle(trace, &request, reason, None);
            values(lua, &value)
        }
    }
}

/// Perform the effect and translate its outcome into a resume value.
fn outcome(
    lua: &Lua,
    trace: &mut Option<Trace>,
    request: &Request,
    result: Result<serde_json::Value, Failure>,
) -> Result<MultiValue, Failure> {
    match result {
        Ok(value) => {
            settle(trace, request, "ok", None);
            values(lua, &value)
        }
        Err(failure) if request.fallible => {
            settle(trace, request, "error", Some(&failure.message));
            let message =
                Value::String(lua.create_string(&failure.message).map_err(Failure::from)?);
            Ok(MultiValue::from_vec(vec![Value::Nil, message]))
        }
        // Delivered as data so `hob.effect` raises inside the coroutine, where
        // the flow can still catch it with `pcall`.
        Err(failure) => {
            settle(trace, request, "error", Some(&failure.message));
            values(lua, &abort(&failure.message))
        }
    }
}

/// Record how the effect ended, when a trace is being kept.
fn settle(trace: &mut Option<Trace>, request: &Request, outcome: &str, error: Option<&str>) {
    if let Some(trace) = trace.as_mut() {
        trace.outcome(request, outcome, error);
    }
}

fn values(lua: &Lua, value: &serde_json::Value) -> Result<MultiValue, Failure> {
    let value = json_to_lua(lua, value).map_err(Failure::from)?;
    Ok(MultiValue::from_vec(vec![value]))
}

fn abort_failure(request: &Request) -> Failure {
    let message = request
        .cmd
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("aborted");
    let code = request
        .cmd
        .get("code")
        .and_then(serde_json::Value::as_u64)
        .and_then(|code| u8::try_from(code).ok())
        .unwrap_or(1);
    Failure::with_code(message, code)
}
