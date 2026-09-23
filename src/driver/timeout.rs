// --- driver::timeout ---
// A run that overstays `--timeout`.
//
// The timer runs on its own thread, like the signal handler, so it fires while
// the main thread is blocked in a read, a wait or a request. It kills the live
// process groups and exits with the code `timeout(1)` uses; the watchdog is
// cancelled when the run ends before its limit.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::exec::proc::Children;

/// Exit code a run over its timeout reports, as `timeout(1)` does.
pub(crate) const EXIT: u8 = 124;

/// Cancels the timer when dropped.
#[derive(Debug)]
pub(crate) struct Watchdog {
    cancel: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Watchdog {
    /// Kill the run's processes and exit once `limit` passes.
    pub(crate) fn start(limit: Duration) -> Self {
        let (cancel, wait) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            if wait.recv_timeout(limit) == Err(RecvTimeoutError::Timeout) {
                Children::global().kill_all();
                std::process::exit(i32::from(EXIT));
            }
        });
        Self {
            cancel,
            thread: Some(thread),
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
