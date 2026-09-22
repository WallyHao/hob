//! Failure paths: an effect the engine refuses, a fallible call, an abort and
//! a malformed request each have to end the flow in exactly one way.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_hob");

struct Flow(PathBuf);

impl Flow {
    fn new(tag: &str, source: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hob-flow-{}-{tag}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        fs::write(dir.join("flow.lua"), source).expect("flow file");
        Self(dir)
    }

    fn run(&self) -> Output {
        Command::new(BIN)
            .arg("run")
            .arg(self.0.join("flow.lua"))
            .output()
            .expect("spawn hob")
    }
}

impl Drop for Flow {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn unknown_effect_fails_the_flow() {
    let flow = Flow::new("unknown", r#"hob.effect("nope", "x")"#);
    let output = flow.run();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("unknown effect `nope.x`"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn fallible_effect_returns_nil_and_message() {
    let flow = Flow::new(
        "fallible",
        r#"
        local ok, err = hob.effect("nope", "x", {}, { fallible = true })
        hob.term.print(tostring(ok) .. "|" .. tostring(err))
        "#,
    );
    let output = flow.run();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "nil|unknown effect `nope.x`\n");
}

#[test]
fn abort_sets_the_exit_code() {
    let flow = Flow::new("abort", r#"hob.abort("stop here", 3)"#);
    let output = flow.run();
    assert_eq!(output.status.code(), Some(3));
    assert!(stderr(&output).contains("stop here"), "{}", stderr(&output));
}

#[test]
fn assert_aborts_with_its_message() {
    let flow = Flow::new("assert", r#"hob.assert(false, "must hold")"#);
    let output = flow.run();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("must hold"), "{}", stderr(&output));
}

#[test]
fn a_malformed_effect_is_a_protocol_error() {
    let flow = Flow::new("malformed", r#"coroutine.yield("not a table")"#);
    let output = flow.run();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("effect must be a table"),
        "{}",
        stderr(&output)
    );
}
