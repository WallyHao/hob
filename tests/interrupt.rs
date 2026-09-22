//! Ctrl-C and timeouts: a run must not leave a process tree behind.

mod common;

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use common::Flow;

#[test]
fn an_interrupt_kills_the_process_group() {
    let flow = Flow::new(
        "interrupt",
        r#"hob.proc.exec{ "sh", "-c", "sleep 30 & echo $! > grand.pid; wait" }"#,
    );
    let mut hob = Command::new(common::BIN)
        .env("HOB_CONFIG_DIR", flow.config())
        .current_dir(flow.dir())
        .arg("run")
        .arg("flow.lua")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hob");
    let grandchild = grandchild(&flow);
    signal(hob.id(), nix::sys::signal::Signal::SIGINT);
    let status = hob.wait().expect("wait for hob");
    assert_eq!(status.code(), Some(130), "{status:?}");
    wait_for(
        || !Path::new(&format!("/proc/{grandchild}")).exists(),
        "the grandchild",
    );
}

#[test]
fn a_timeout_kills_the_process_group() {
    let flow = Flow::new(
        "timeout-group",
        r#"
        local result = hob.proc.exec(
          { "sh", "-c", "sleep 30 & echo $! > grand.pid; wait" },
          { timeout_ms = 500 }
        )
        hob.term.print(tostring(result.code))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert!(common::stdout(&output).contains("124"), "{output:?}");
    let grandchild = grandchild(&flow);
    wait_for(
        || !Path::new(&format!("/proc/{grandchild}")).exists(),
        "the grandchild",
    );
}

/// Wait until the command has started, then read the grandchild it wrote down.
fn grandchild(flow: &Flow) -> i32 {
    let path = flow.dir().join("grand.pid");
    wait_for(|| path.exists(), "the command to start");
    std::fs::read_to_string(&path)
        .expect("pid file")
        .trim()
        .parse()
        .expect("a pid")
}

fn signal(pid: u32, signal: nix::sys::signal::Signal) {
    let pid = i32::try_from(pid).expect("a pid fits in i32");
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), signal).expect("signal hob");
}

/// Poll a condition for a moment, so the test is not a race.
fn wait_for(condition: impl Fn() -> bool, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {what}");
}
