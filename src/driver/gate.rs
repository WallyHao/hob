// --- driver::gate ---
// The decision in front of every effect: perform it, ask about it, or refuse it
// and stand in for its result.
//
// `--dry-run` performs reads and refuses everything else; `--step` asks before
// every effect. A refusal is announced with a one-line description rather than
// the raw arguments, and values the environment calls secret are masked in that
// line, so a preview cannot leak a key the way a dump of the request would.

use std::io::{self, Write as _};

use serde_json::Value;

use crate::effect::{Failure, Request};
use crate::exec::redact::Redactor;
use crate::exec::{State, describe, safety::Safety, skipped};

/// How much of the work to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Everything.
    Run,
    /// Reads only; the rest is refused and printed.
    DryRun,
    /// Ask before every effect.
    Step,
}

/// The control flags one run carries.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Control {
    /// How much of the work to perform.
    pub(crate) mode: Mode,
    /// Log level: 0 quiet, 1 normal, 2 debug, 3 trace.
    pub(crate) verbosity: u8,
    /// Answer questions with their default instead of reading stdin.
    pub(crate) yes: bool,
}

/// What to do with one effect.
#[derive(Debug)]
pub(crate) enum Decision {
    /// Perform it.
    Run,
    /// Resume the flow with this value instead, because `reason` refused it.
    Skip {
        /// `dry-run` or `declined`, recorded in the trace and printed.
        reason: &'static str,
        /// The conservative answer the flow sees.
        value: Value,
    },
}

/// The preview state.
#[derive(Debug)]
pub(crate) struct Gate {
    mode: Mode,
    yes: bool,
    /// Built only when a preview prints; a plain run never pays for it.
    redact: Option<Redactor>,
}

impl Gate {
    /// A gate for one run.
    pub(crate) fn new(control: Control) -> Self {
        Self {
            mode: control.mode,
            yes: control.yes,
            redact: (control.mode != Mode::Run).then(Redactor::from_env),
        }
    }

    /// Decide what happens to one effect.
    pub(crate) fn decide(&self, state: &State, request: &Request) -> Result<Decision, Failure> {
        let safety = Safety::of(&request.ns, &request.op);
        match self.mode {
            Mode::Run => Ok(Decision::Run),
            Mode::DryRun if safety == Safety::ReadOnly => Ok(Decision::Run),
            Mode::DryRun => Ok(self.refuse(state, request, "dry-run")),
            Mode::Step => {
                if self.yes || self.approve(request)? {
                    Ok(Decision::Run)
                } else {
                    Ok(self.refuse(state, request, "declined"))
                }
            }
        }
    }

    /// Print the effect and stand in for its result.
    fn refuse(&self, state: &State, request: &Request, reason: &'static str) -> Decision {
        eprintln!("{reason}: {}", self.line(request));
        Decision::Skip {
            reason,
            value: skipped::result(state, request),
        }
    }

    /// Show the effect and ask; an empty line means yes.
    fn approve(&self, request: &Request) -> Result<bool, Failure> {
        let mut err = io::stderr();
        let _ = write!(err, "step: {}\nperform? [Y/n] ", self.line(request));
        let _ = err.flush();
        let mut line = String::new();
        let read = io::stdin()
            .read_line(&mut line)
            .map_err(|error| Failure::new(format!("cannot read input: {error}")))?;
        if read == 0 {
            return Err(Failure::new("input closed before an answer arrived"));
        }
        Ok(!matches!(
            line.trim().to_ascii_lowercase().as_str(),
            "n" | "no"
        ))
    }

    /// One line describing the effect, with secrets masked.
    fn line(&self, request: &Request) -> String {
        let line = describe::line(request);
        match &self.redact {
            Some(redactor) => redactor.text(&line),
            None => line,
        }
    }
}
