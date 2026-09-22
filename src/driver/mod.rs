// --- driver ---
// The coroutine loop: the only place that resumes a Lua thread and the only
// place that decides what an effect means. Keeping both here is what makes
// every side effect pass through one choke point, so a preview or a trace is
// implemented once rather than at each call site.

use mlua::thread::ThreadStatus;
use mlua::{Lua, MultiValue, Table, Value};

use crate::effect::{Failure, Request, abort, json_to_lua};
use crate::exec;
use crate::lua::{self, preload, pure};
use crate::paths::Paths;

/// Run a flow to completion. `name` is the command name the flow sees.
pub(crate) fn run(name: &str, source: &str, args: &[String]) -> Result<(), Failure> {
    let lua = lua::new_vm().map_err(Failure::from)?;
    let hob = preload::install(&lua).map_err(Failure::from)?;
    pure::install(&lua, &hob).map_err(Failure::from)?;
    publish(&lua, &hob, name, args).map_err(Failure::from)?;
    let mut state = exec::State::new(Paths::resolve());

    let body = lua
        .load(source)
        .set_name(name)
        .into_function()
        .map_err(Failure::from)?;
    let thread = lua.create_thread(body).map_err(Failure::from)?;
    let mut resume = MultiValue::new();
    loop {
        let yielded = thread.resume::<MultiValue>(resume).map_err(Failure::from)?;
        // A coroutine that finished and one that yielded both return from
        // `resume`, so the status is the only thing that tells them apart.
        if !matches!(thread.status(), ThreadStatus::Resumable) {
            return Ok(());
        }
        resume = step(&lua, &mut state, &yielded)?;
    }
}

/// Handle one yielded effect.
fn step(lua: &Lua, state: &mut exec::State, yielded: &MultiValue) -> Result<MultiValue, Failure> {
    let first = yielded
        .front()
        .ok_or_else(|| Failure::new("the flow yielded no value"))?;
    let request = Request::from_lua(first)?;
    // `abort` is control flow, not work: the flow is over the moment it is
    // yielded, and nothing after it in the script ever runs.
    if request.ns == "term" && request.op == "abort" {
        return Err(abort_failure(&request));
    }
    match exec::perform(state, &request) {
        Ok(value) => values(lua, &value),
        Err(failure) if request.fallible => {
            let message =
                Value::String(lua.create_string(&failure.message).map_err(Failure::from)?);
            Ok(MultiValue::from_vec(vec![Value::Nil, message]))
        }
        // Delivered as data so `hob.effect` raises inside the coroutine, where
        // the flow can still catch it with `pcall`.
        Err(failure) => values(lua, &abort(&failure.message)),
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

/// Publish the invocation before the body runs, so a module can read it
/// without threading it through every call.
fn publish(lua: &Lua, hob: &Table, name: &str, args: &[String]) -> mlua::Result<()> {
    let table = lua.create_table_with_capacity(args.len(), 0)?;
    for (index, arg) in args.iter().enumerate() {
        table.raw_set(index + 1, arg.as_str())?;
    }
    hob.set("args", table)?;
    hob.set("command", name)?;
    Ok(())
}
