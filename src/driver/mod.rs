// --- driver ---
// The coroutine loop: the only place that resumes a Lua thread and the only
// place that decides what an effect means. Keeping both here is what makes
// every side effect pass through one choke point, so a preview, a step or a
// trace is implemented once rather than at each call site.

pub(crate) mod gate;
mod step;

use std::path::PathBuf;

use mlua::thread::ThreadStatus;
use mlua::{Lua, MultiValue, Table};

use crate::cli::trace::Trace;
use crate::effect::Failure;
use crate::exec;
use crate::lua::{self, preload, pure};
use crate::paths::Paths;

pub(crate) use gate::{Control, Mode};

/// Run a flow to completion. `name` is the command name the flow sees.
pub(crate) fn run(
    name: &str,
    source: &str,
    args: &[String],
    control: Control,
    trace: Option<PathBuf>,
) -> Result<(), Failure> {
    let lua = lua::new_vm().map_err(Failure::from)?;
    let hob = preload::install(&lua).map_err(Failure::from)?;
    pure::install(&lua, &hob).map_err(Failure::from)?;
    publish(&lua, &hob, name, args).map_err(Failure::from)?;
    let mut state = exec::State::new(Paths::resolve(), control.verbosity, control.yes);
    let mut gate = gate::Gate::new(control);
    let mut trace = trace.map(|path| Trace::create(&path)).transpose()?;

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
        resume = step::handle(&lua, &mut state, &mut gate, &mut trace, &yielded)?;
    }
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
