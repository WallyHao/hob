// --- effect::operation ---
// Decode once before the gate; exhaustive matches keep each policy in step.

use super::{
    Failure, Request,
    ops::{agent, file, logs, proc, term, tmpl},
};
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub(crate) enum Operation {
    FileRead(file::Read),
    FileWrite(file::Write),
    FileStat(file::Stat),
    FileList(file::List),
    ProcOpen(proc::Open),
    ProcExec(proc::Exec),
    ProcShell(proc::Shell),
    ProcWhich(proc::Which),
    ProcSetenv(proc::SetEnv),
    ProcUnset(proc::Unset),
    ProcChdir(proc::Chdir),
    ProcSetup(proc::Setup),
    ProcState(proc::Handle),
    ProcReset(proc::Handle),
    ProcClose(proc::Handle),
    AgentAsk(agent::Ask),
    AgentOpen(agent::Open),
    AgentList(agent::List),
    AgentSend(agent::Send),
    AgentPush(agent::Push),
    AgentTurns(agent::Handle),
    AgentUsage(agent::Handle),
    AgentReset(agent::Handle),
    AgentClose(agent::Handle),
    TermPrint(term::Print),
    TermInput(term::Input),
    TermAllow(term::Allow),
    TermSelect(term::Select),
    TermChoose(term::Choose),
    LogsWrite(logs::Write),
    TmplFetch(tmpl::Fetch),
}

impl Operation {
    pub(crate) fn parse(request: &Request) -> Result<Self, Failure> {
        match (request.ns.as_str(), request.op.as_str()) {
            ("file", "read") => decode(request).map(Self::FileRead),
            ("file", "write") => decode(request).map(Self::FileWrite),
            ("file", "stat") => decode(request).map(Self::FileStat),
            ("file", "list") => decode(request).map(Self::FileList),
            ("proc", "open") => decode(request).map(Self::ProcOpen),
            ("proc", "exec") => decode(request).map(Self::ProcExec),
            ("proc", "shell") => decode(request).map(Self::ProcShell),
            ("proc", "which") => decode(request).map(Self::ProcWhich),
            ("proc", "setenv") => decode(request).map(Self::ProcSetenv),
            ("proc", "unset") => decode(request).map(Self::ProcUnset),
            ("proc", "chdir") => decode(request).map(Self::ProcChdir),
            ("proc", "setup") => decode(request).map(Self::ProcSetup),
            ("proc", "state") => decode(request).map(Self::ProcState),
            ("proc", "reset") => decode(request).map(Self::ProcReset),
            ("proc", "close") => decode(request).map(Self::ProcClose),
            ("agent", "ask") => decode(request).map(Self::AgentAsk),
            ("agent", "open") => decode(request).map(Self::AgentOpen),
            ("agent", "list") => decode(request).map(Self::AgentList),
            ("agent", "send") => decode(request).map(Self::AgentSend),
            ("agent", "push") => decode(request).map(Self::AgentPush),
            ("agent", "turns") => decode(request).map(Self::AgentTurns),
            ("agent", "usage") => decode(request).map(Self::AgentUsage),
            ("agent", "reset") => decode(request).map(Self::AgentReset),
            ("agent", "close") => decode(request).map(Self::AgentClose),
            ("term", "print") => decode(request).map(Self::TermPrint),
            ("term", "input") => decode(request).map(Self::TermInput),
            ("term", "allow") => decode(request).map(Self::TermAllow),
            ("term", "select") => decode(request).map(Self::TermSelect),
            ("term", "choose") => decode(request).map(Self::TermChoose),
            ("logs", "write") => decode(request).map(Self::LogsWrite),
            ("tmpl", "fetch") => decode(request).map(Self::TmplFetch),
            _ => Err(Failure::unknown(&request.ns, &request.op)),
        }
    }
}

fn decode<T: DeserializeOwned>(request: &Request) -> Result<T, Failure> {
    let mut arguments = request.cmd.clone();
    // Empty unmarked Lua tables remain convenient for declared list arguments.
    let lists: &[&str] = match request.ns.as_str() {
        "agent" => &["messages", "tools"],
        "proc" => &["argv"],
        "term" => &["options", "defaults"],
        _ => &[],
    };
    for key in lists {
        if let Some(value) = arguments.get_mut(*key)
            && value.as_object().is_some_and(serde_json::Map::is_empty)
        {
            *value = serde_json::Value::Array(Vec::new());
        }
    }
    serde_json::from_value(arguments)
        .map_err(|error| Failure::new(format!("`{}.{}`: {error}", request.ns, request.op)))
}
