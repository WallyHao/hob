// --- exec ---
// Where effects are performed. Dispatch is by `ns.op` and nothing else, so an
// unimplemented operation is a named error rather than a silent no-op.
//
// `RunContext` is everything that lives as long as one flow: open sessions, the
// directories templates resolve against, and the control flags that shape what
// a run may do. It is threaded through every call rather than kept global, so
// one run stays a value instead of an ambient.

pub(crate) mod agent;
pub(crate) mod budget;
pub(crate) mod describe;
pub(crate) mod file;
pub(crate) mod logs;
pub(crate) mod proc;
pub(crate) mod redact;
pub(crate) mod skipped;
pub(crate) mod term;
pub(crate) mod tmpl;

mod context;
use crate::effect::{Failure, Operation};
pub(crate) use context::RunContext;
use serde_json::Value;

/// Execute a validated request; a new operation must be handled here.
pub(crate) fn perform(state: &mut RunContext, request: &Operation) -> Result<Value, Failure> {
    match request {
        Operation::FileRead(args) => file::read(args),
        Operation::FileWrite(args) => file::write(args),
        Operation::FileStat(args) => file::stat(args),
        Operation::FileList(args) => file::list(args),
        Operation::ProcOpen(args) => proc::open(state, args),
        Operation::ProcExec(args) => proc::exec(state, args),
        Operation::ProcShell(args) => proc::shell(state, args),
        Operation::ProcWhich(args) => proc::which(args),
        Operation::ProcSetenv(args) => proc::setenv(state, args),
        Operation::ProcUnset(args) => proc::unset(state, args),
        Operation::ProcChdir(args) => proc::chdir(state, args),
        Operation::ProcSetup(args) => proc::setup(state, args),
        Operation::ProcState(args) => proc::state_of(state, args),
        Operation::ProcReset(args) => proc::reset(state, args),
        Operation::ProcClose(args) => proc::close(state, args),
        Operation::AgentAsk(args) => agent::ask(state, args),
        Operation::AgentOpen(args) => agent::open(state, args),
        Operation::AgentList(args) => agent::list(state, args),
        Operation::AgentSend(args) => agent::send(state, args),
        Operation::AgentPush(args) => agent::push(state, args),
        Operation::AgentTurns(args) => agent::turns(state, args),
        Operation::AgentUsage(args) => agent::usage(state, args),
        Operation::AgentReset(args) => agent::reset(state, args),
        Operation::AgentClose(args) => agent::close(state, args),
        Operation::TermPrint(args) => term::print(args, state.color),
        Operation::TermInput(args) => term::input(args, state.yes),
        Operation::TermAllow(args) => term::allow(args, state.yes),
        Operation::TermSelect(args) => term::select(args, state.yes),
        Operation::TermChoose(args) => term::choose(args, state.yes),
        Operation::LogsWrite(args) => logs::write(state, args),
        Operation::TmplFetch(args) => tmpl::fetch(&state.paths, args),
    }
}
