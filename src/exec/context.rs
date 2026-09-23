// --- exec::context ---
// Mutable resources belong to one invocation.

use super::{
    agent,
    budget::{Budget, Limits},
    proc, term,
};
use crate::paths::Paths;
use crate::provider::Cache;
use std::collections::HashMap;

/// Per-flow state.
#[derive(Debug)]
pub(crate) struct RunContext {
    /// Directories a flow may read templates from.
    pub(crate) paths: Paths,
    /// Open `proc` sessions by handle.
    pub(crate) procs: HashMap<u64, proc::Session>,
    /// Open `agent` conversations by handle.
    pub(crate) chats: HashMap<u64, agent::Chat>,
    /// Provider clients, one per provider, for the life of the run.
    pub(crate) clients: Cache,
    /// Live process groups, shared with the interrupt handler.
    pub(crate) children: proc::Children,
    /// Log level: 0 quiet, 1 normal, 2 debug, 3 trace.
    pub(crate) verbosity: u8,
    /// Answer questions with their default instead of reading stdin.
    pub(crate) yes: bool,
    /// What the run may spend on model calls.
    pub(crate) budget: Budget,
    /// How `term.print` treats styling.
    pub(crate) color: term::Color,
    next_id: u64,
}

impl RunContext {
    /// Mutable resources for one run.
    pub(crate) fn new(
        paths: Paths,
        verbosity: u8,
        yes: bool,
        children: proc::Children,
        limits: Limits,
        color: term::Color,
    ) -> Self {
        Self {
            paths,
            procs: HashMap::new(),
            chats: HashMap::new(),
            clients: Cache::default(),
            children,
            verbosity,
            yes,
            budget: Budget::new(limits),
            color,
            next_id: 0,
        }
    }

    /// A fresh handle id, unique within the run.
    pub(crate) fn id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}
