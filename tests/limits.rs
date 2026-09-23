//! Engine limits: a flow cannot take the process down with it.

mod common;

use std::process::{Command, Stdio};

use common::{BIN, Flow, stderr};

#[test]
fn printing_to_a_closed_stdout_does_not_panic() {
    // The reader goes away before the flow writes, which is what `| head`
    // does; the flow writes past the pipe buffer so a broken pipe is certain.
    let flow = Flow::new(
        "closed-stdout",
        r#"for i = 1, 10000 do hob.term.print("line " .. i) end"#,
    );
    let mut child = Command::new(BIN)
        .env("HOB_CONFIG_DIR", flow.config())
        .current_dir(flow.dir())
        .arg("run")
        .arg("flow.lua")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hob");
    drop(child.stdout.take());
    let output = child.wait_with_output().expect("wait for hob");
    assert!(output.status.success(), "{output:?}");
    assert!(!stderr(&output).contains("panicked"), "{output:?}");
}

#[test]
fn a_runaway_flow_fails_on_the_memory_limit() {
    let flow = Flow::new(
        "memory",
        r#"local chunks = {} while true do chunks[#chunks + 1] = string.rep("x", 1024 * 1024) end"#,
    );
    let output = flow.run(&[]);
    assert!(!output.status.success(), "{output:?}");
    assert!(stderr(&output).contains("memory"), "{output:?}");
}
