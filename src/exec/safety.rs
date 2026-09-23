// --- exec::safety ---
// Whether a preview may perform an effect.
//
// The classification is the engine's, not the flow's: a flow that could mark
// its own writes as reads would turn `--dry-run` into a suggestion. An
// operation that is not listed is a side effect, so a new effect is refused by
// a preview until someone classifies it here.

/// What an effect may do to the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Safety {
    /// Observes, or touches only engine state.
    ReadOnly,
    /// Writes, starts a process, spends money or needs a person.
    SideEffect,
}

impl Safety {
    /// Classify one operation.
    pub(crate) fn of(ns: &str, op: &str) -> Self {
        if READS.contains(&(ns, op)) {
            Self::ReadOnly
        } else {
            Self::SideEffect
        }
    }
}

/// Effects a preview may perform: they observe, or change only in-memory state
/// the run itself owns.
const READS: &[(&str, &str)] = &[
    ("agent", "open"),
    ("agent", "push"),
    ("agent", "turns"),
    ("agent", "usage"),
    ("agent", "reset"),
    ("agent", "close"),
    ("file", "read"),
    ("file", "stat"),
    ("file", "list"),
    ("logs", "write"),
    ("proc", "open"),
    ("proc", "which"),
    ("proc", "setenv"),
    ("proc", "unset"),
    ("proc", "chdir"),
    ("proc", "setup"),
    ("proc", "state"),
    ("proc", "reset"),
    ("proc", "close"),
    ("term", "print"),
    ("tmpl", "fetch"),
];

#[cfg(test)]
mod tests {
    use super::Safety;

    #[test]
    fn writes_and_processes_are_side_effects() {
        assert_eq!(Safety::of("file", "write"), Safety::SideEffect);
        assert_eq!(Safety::of("proc", "exec"), Safety::SideEffect);
        assert_eq!(Safety::of("proc", "shell"), Safety::SideEffect);
        assert_eq!(Safety::of("agent", "ask"), Safety::SideEffect);
        assert_eq!(Safety::of("agent", "send"), Safety::SideEffect);
        // Listing models reaches the network and reads a key, so a preview
        // must not perform it even though it writes nothing.
        assert_eq!(Safety::of("agent", "list"), Safety::SideEffect);
        assert_eq!(Safety::of("term", "allow"), Safety::SideEffect);
    }

    #[test]
    fn reads_are_previewable() {
        assert_eq!(Safety::of("file", "read"), Safety::ReadOnly);
        assert_eq!(Safety::of("tmpl", "fetch"), Safety::ReadOnly);
        assert_eq!(Safety::of("proc", "setenv"), Safety::ReadOnly);
        assert_eq!(Safety::of("agent", "usage"), Safety::ReadOnly);
    }

    #[test]
    fn an_unknown_operation_is_not_previewable() {
        assert_eq!(Safety::of("file", "remove"), Safety::SideEffect);
        assert_eq!(Safety::of("future", "thing"), Safety::SideEffect);
    }
}
