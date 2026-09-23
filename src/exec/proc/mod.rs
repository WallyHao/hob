// --- exec::proc ---
// Subprocesses and sessions. One-shot calls open a default session and throw
// it away; a handle from `open` keeps the working directory, the environment
// overrides and the profile between commands.

mod children;
pub(crate) mod run;
pub(crate) mod session;
pub(crate) mod spawn;

use std::collections::BTreeMap;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::proc::{
    Chdir, Exec, Handle, Open, Options, SetEnv, Setup, Shell, Unset, Which,
};
use crate::exec::RunContext;

pub(crate) use children::Children;
pub(crate) use session::Session;

/// Open a session and return its handle.
pub(crate) fn open(state: &mut RunContext, request: &Open) -> Result<Value, Failure> {
    let session = Session::open(
        request.cwd.clone(),
        request.env.clone(),
        request.profile.clone(),
        request.env_clear,
    )?;
    let id = state.id();
    state.procs.insert(id, session);
    Ok(Value::from(id))
}

/// Run one program.
pub(crate) fn exec(state: &mut RunContext, request: &Exec) -> Result<Value, Failure> {
    with(state, &request.options, |state, session, options| {
        run::program(state, session, &request.argv, options)
    })
}

/// Run one shell line.
pub(crate) fn shell(state: &mut RunContext, request: &Shell) -> Result<Value, Failure> {
    with(state, &request.options, |state, session, options| {
        run::shell(state, session, &request.line, options)
    })
}

/// Resolve a program on `PATH`.
pub(crate) fn which(request: &Which) -> Result<Value, Failure> {
    spawn::which(&request.prog)
}

/// Add or replace one environment override.
pub(crate) fn setenv(state: &mut RunContext, request: &SetEnv) -> Result<Value, Failure> {
    session(state, request.session)?.setenv(&request.name, &request.value);
    Ok(Value::Null)
}

/// Remove one override.
pub(crate) fn unset(state: &mut RunContext, request: &Unset) -> Result<Value, Failure> {
    session(state, request.session)?.unset(&request.name);
    Ok(Value::Null)
}

/// Move the session.
pub(crate) fn chdir(state: &mut RunContext, request: &Chdir) -> Result<Value, Failure> {
    session(state, request.session)?.chdir(&request.path)?;
    Ok(Value::Null)
}

/// Replace the profile.
pub(crate) fn setup(state: &mut RunContext, request: &Setup) -> Result<Value, Failure> {
    session(state, request.session)?.setup(&request.text);
    Ok(Value::Null)
}

/// Report where the session is now.
pub(crate) fn state_of(state: &RunContext, request: &Handle) -> Result<Value, Failure> {
    Ok(session_ref(state, request.session)?.state())
}

/// Restore the opening context.
pub(crate) fn reset(state: &mut RunContext, request: &Handle) -> Result<Value, Failure> {
    session(state, request.session)?.reset();
    Ok(Value::Null)
}

/// Drop the session.
pub(crate) fn close(state: &mut RunContext, request: &Handle) -> Result<Value, Failure> {
    state
        .procs
        .remove(&request.session)
        .ok_or_else(|| unknown(request.session))?;
    Ok(Value::Null)
}

/// Run against a session, or against a fresh default one for a one-shot call.
fn with<T>(
    state: &RunContext,
    options: &Options,
    call: impl FnOnce(&RunContext, &Session, &Options) -> Result<T, Failure>,
) -> Result<T, Failure> {
    match options.session {
        Some(id) => call(state, session_ref(state, id)?, options),
        None => call(
            state,
            &Session::open(None, BTreeMap::new(), None, options.env_clear)?,
            options,
        ),
    }
}

fn session_ref(state: &RunContext, id: u64) -> Result<&Session, Failure> {
    state.procs.get(&id).ok_or_else(|| unknown(id))
}

fn session(state: &mut RunContext, id: u64) -> Result<&mut Session, Failure> {
    state.procs.get_mut(&id).ok_or_else(|| unknown(id))
}

fn unknown(id: u64) -> Failure {
    Failure::new(format!("no session with handle {id}"))
}
