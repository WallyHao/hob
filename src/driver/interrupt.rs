// --- driver::interrupt ---
// Ctrl-C ends the run now.
//
// The handler runs on its own thread, so it works while the main thread is
// blocked in a read, a wait or a request. It kills every live process group
// first, which is what keeps an interrupted command from leaving orphans, then
// exits with the conventional 130. Nothing needs unwinding: a trace flushes
// every line it writes and the rest of the state dies with the process.

use crate::exec::proc::Children;

/// Watch for Ctrl-C for the rest of the process's life.
pub(crate) fn install(children: Children) {
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            // Without a runtime the default disposition still terminates the
            // process; only the children would be left behind.
            return;
        };
        runtime.block_on(async {
            if tokio::signal::ctrl_c().await.is_ok() {
                children.kill_all();
                std::process::exit(130);
            }
        });
    });
}
