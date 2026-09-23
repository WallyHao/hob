// --- driver::interrupt ---
// Ctrl-C and SIGTERM end the run now.
//
// The handler runs on its own thread, so it works while the main thread is
// blocked in a read, a wait or a request. It kills every live process group
// first, which is what keeps an interrupted command from leaving orphans, then
// exits with the conventional 128+signal. Nothing needs unwinding: a trace
// flushes every line it writes and the rest of the state dies with the process.

use std::sync::OnceLock;

use crate::exec::proc::Children;

/// Install the handler once for the rest of the process's life.
pub(crate) fn install() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::thread::spawn(|| {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                // Without a runtime the default disposition still terminates the
                // process; only the children would be left behind.
                return;
            };
            runtime.block_on(async {
                if let Some(code) = caught().await {
                    Children::global().kill_all();
                    std::process::exit(code);
                }
            });
        });
    });
}

/// The exit code of the signal that arrived, or `None` when none can be
/// watched and the default disposition has to stay in charge.
#[cfg(unix)]
async fn caught() -> Option<i32> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut interrupt = signal(SignalKind::interrupt()).ok()?;
    let mut terminate = signal(SignalKind::terminate()).ok()?;
    let code = tokio::select! {
        _ = interrupt.recv() => 130,
        _ = terminate.recv() => 143,
    };
    Some(code)
}

#[cfg(not(unix))]
async fn caught() -> Option<i32> {
    tokio::signal::ctrl_c().await.ok().map(|()| 130)
}
