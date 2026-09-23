// --- effect::policy ---
// Every operation explicitly declares its preview and trace content policy.
use super::Operation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Safety {
    ReadOnly,
    LocalState,
    SideEffect,
}

impl Operation {
    pub(crate) fn policy(&self) -> (Safety, &'static [&'static str]) {
        use Safety::{LocalState, ReadOnly, SideEffect};
        match self {
            Self::FileWrite(..) => (SideEffect, &["text"]),
            Self::ProcOpen(..) => (LocalState, &["profile", "env"]),
            Self::ProcExec(..) => (SideEffect, &["stdin"]),
            Self::ProcShell(..) => (SideEffect, &["stdin", "line"]),
            Self::ProcSetenv(..) => (LocalState, &["value"]),
            Self::ProcSetup(..) => (LocalState, &["text"]),
            Self::AgentAsk(..) | Self::AgentSend(..) => {
                (SideEffect, &["prompt", "system", "messages"])
            }
            Self::AgentOpen(..) => (LocalState, &["system"]),
            Self::AgentPush(..) => (LocalState, &["message"]),
            Self::TermPrint(..) => (ReadOnly, &["text"]),
            Self::LogsWrite(..) => (ReadOnly, &["msg"]),
            Self::FileRead(..)
            | Self::FileStat(..)
            | Self::FileList(..)
            | Self::ProcWhich(..)
            | Self::ProcState(..)
            | Self::AgentTurns(..)
            | Self::AgentUsage(..)
            | Self::TmplFetch(..) => (ReadOnly, &[]),
            Self::ProcUnset(..)
            | Self::ProcChdir(..)
            | Self::ProcReset(..)
            | Self::ProcClose(..)
            | Self::AgentReset(..)
            | Self::AgentClose(..) => (LocalState, &[]),
            Self::AgentList(..)
            | Self::TermInput(..)
            | Self::TermAllow(..)
            | Self::TermSelect(..)
            | Self::TermChoose(..) => (SideEffect, &[]),
        }
    }
}
