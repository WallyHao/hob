// --- exec::proc::children ---
// The process groups a run has alive.
//
// Every child is spawned as its own group leader, so a command that starts a
// tree (`sh -c 'long & wait'`) is killed whole rather than leaving orphans
// behind. The registry exists so the interrupt handler, which runs on another
// thread, can kill what is running when the user presses Ctrl-C.

use std::sync::{Arc, Mutex};

/// The live process groups of one run.
#[derive(Clone, Debug, Default)]
pub(crate) struct Children {
    pids: Arc<Mutex<Vec<u32>>>,
}

impl Children {
    /// An empty registry.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Register a group leader until the guard is dropped.
    pub(crate) fn track(&self, pid: u32) -> Tracked {
        if let Ok(mut pids) = self.pids.lock() {
            pids.push(pid);
        }
        Tracked {
            children: self.clone(),
            pid,
        }
    }

    /// Kill every live group with `SIGKILL`, ignoring failures.
    pub(crate) fn kill_all(&self) {
        let Ok(pids) = self.pids.lock() else {
            return;
        };
        for pid in pids.iter() {
            kill_group(*pid);
        }
    }

    fn forget(&self, pid: u32) {
        if let Ok(mut pids) = self.pids.lock() {
            pids.retain(|live| *live != pid);
        }
    }
}

/// Removes its process from the registry when dropped.
#[derive(Debug)]
pub(crate) struct Tracked {
    children: Children,
    pid: u32,
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.children.forget(self.pid);
    }
}

/// `SIGKILL` one process group.
///
/// A pid is its own group id because every child is spawned with
/// `process_group(0)`.
#[cfg(unix)]
pub(crate) fn kill_group(pid: u32) {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid;
    let Ok(pid) = i32::try_from(pid) else {
        return;
    };
    let _ = killpg(Pid::from_raw(pid), Signal::SIGKILL);
}

#[cfg(not(unix))]
pub(crate) fn kill_group(_pid: u32) {}

#[cfg(test)]
mod tests {
    use super::Children;

    #[test]
    fn a_tracked_group_is_forgotten_when_the_guard_drops() {
        let children = Children::new();
        let guard = children.track(42);
        assert_eq!(children.pids.lock().expect("lock").len(), 1);
        drop(guard);
        assert!(children.pids.lock().expect("lock").is_empty());
    }
}
