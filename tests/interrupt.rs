//! Signals and timeouts: a run must not leave a process tree behind.

mod common;

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use common::Flow;
use nix::sys::signal::Signal;

#[test]
fn an_interrupt_kills_the_process_group() {
    kills_the_group_on("interrupt", Signal::SIGINT, 130);
}

#[test]
fn a_terminate_kills_the_process_group() {
    kills_the_group_on("terminate", Signal::SIGTERM, 143);
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

#[test]
fn a_flow_timeout_stops_the_run() {
    let flow = Flow::new("flow-timeout", "while true do end");
    let output = flow.run(&["--timeout", "1"]);
    assert_eq!(output.status.code(), Some(124), "{output:?}");
}

#[test]
fn a_flow_timeout_does_not_wait_for_itself() {
    let flow = Flow::new("flow-timeout-cancel", r#"hob.term.print("done")"#);
    let output = flow.run(&["--timeout", "30"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(common::stdout(&output), "done\n");
}

/// Start a flow that waits on a grandchild, signal it, and check both are gone.
fn kills_the_group_on(tag: &str, signal: Signal, code: i32) {
    let flow = Flow::new(
        tag,
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
    send(hob.id(), signal);
    let status = hob.wait().expect("wait for hob");
    assert_eq!(status.code(), Some(code), "{status:?}");
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

fn send(pid: u32, signal: Signal) {
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
