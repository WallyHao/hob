// --- exec ---
// Where effects are performed. Dispatch is by `ns.op` and nothing else, so an
// unimplemented operation is a named error rather than a silent no-op.
//
// `State` is everything that lives as long as one flow: open sessions and the
// directories templates resolve against. It is threaded through every call
// rather than kept global, so one run stays a value instead of an ambient.

pub(crate) mod agent;
pub(crate) mod file;
pub(crate) mod logs;
pub(crate) mod proc;
pub(crate) mod term;
pub(crate) mod tmpl;

use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::effect::{Failure, Request};
use crate::paths::Paths;

/// Per-flow state.
#[derive(Debug)]
pub(crate) struct State {
    /// Directories a flow may read templates from.
    pub(crate) paths: Paths,
    /// Open `proc` sessions by handle.
    pub(crate) procs: HashMap<u64, proc::Session>,
    /// Open `agent` conversations by handle.
    pub(crate) chats: HashMap<u64, agent::Chat>,
    next_id: u64,
}

impl State {
    /// State for one run.
    pub(crate) fn new(paths: Paths) -> Self {
        Self {
            paths,
            procs: HashMap::new(),
            chats: HashMap::new(),
            next_id: 0,
        }
    }

    /// A fresh handle id, unique within the run.
    pub(crate) fn id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

/// Perform one effect.
pub(crate) fn perform(state: &mut State, request: &Request) -> Result<Value, Failure> {
    match (request.ns.as_str(), request.op.as_str()) {
        ("agent", "ask") => agent::ask(&decode(request)?),
        ("agent", "open") => agent::open(state, &decode(request)?),
        ("agent", "list") => agent::list(&decode(request)?),
        ("agent", "send") => agent::send(state, &decode(request)?),
        ("agent", "push") => agent::push(state, &decode(request)?),
        ("agent", "turns") => agent::turns(state, &decode(request)?),
        ("agent", "usage") => agent::usage(state, &decode(request)?),
        ("agent", "reset") => agent::reset(state, &decode(request)?),
        ("agent", "close") => agent::close(state, &decode(request)?),
        ("file", "read") => file::read(&decode(request)?),
        ("file", "write") => file::write(&decode(request)?),
        ("file", "stat") => file::stat(&decode(request)?),
        ("file", "list") => file::list(&decode(request)?),
        ("logs", "write") => logs::write(&decode(request)?),
        ("proc", "open") => proc::open(state, &decode(request)?),
        ("proc", "exec") => proc::exec(state, &decode(request)?),
        ("proc", "shell") => proc::shell(state, &decode(request)?),
        ("proc", "which") => proc::which(&decode(request)?),
        ("proc", "setenv") => proc::setenv(state, &decode(request)?),
        ("proc", "unset") => proc::unset(state, &decode(request)?),
        ("proc", "chdir") => proc::chdir(state, &decode(request)?),
        ("proc", "setup") => proc::setup(state, &decode(request)?),
        ("proc", "state") => proc::state_of(state, &decode(request)?),
        ("proc", "reset") => proc::reset(state, &decode(request)?),
        ("proc", "close") => proc::close(state, &decode(request)?),
        ("term", "print") => term::print(&decode(request)?),
        ("term", "input") => term::input(&decode(request)?),
        ("term", "allow") => term::allow(&decode(request)?),
        ("term", "select") => term::select(&decode(request)?),
        ("term", "choose") => term::choose(&decode(request)?),
        ("tmpl", "fetch") => tmpl::fetch(&state.paths, &decode(request)?),
        _ => Err(Failure::unknown(&request.ns, &request.op)),
    }
}

fn decode<T: DeserializeOwned>(request: &Request) -> Result<T, Failure> {
    serde_json::from_value(request.cmd.clone())
        .map_err(|error| Failure::new(format!("`{}.{}`: {error}", request.ns, request.op)))
}
