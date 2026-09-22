//! Happy paths through the real binary: a Lua file goes in, terminal output
//! and exit codes come out.

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

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .arg("run")
            .arg(self.0.join("flow.lua"))
            .args(args)
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
fn print_reaches_stdout() {
    let flow = Flow::new("print", r#"hob.term.print("hello from a flow")"#);
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello from a flow\n");
}

#[test]
fn json_round_trips() {
    let flow = Flow::new(
        "json",
        r#"
        local value = hob.json.decode('{"a": [1, 2]}')
        hob.term.print(tostring(value.a[2]))
        hob.term.print(hob.json.encode({ ok = true }))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "2\n{\"ok\":true}\n");
}

#[test]
fn tmpl_render_substitutes() {
    let flow = Flow::new(
        "tmpl",
        r#"hob.term.print(hob.tmpl.render("hi {name}", { name = "wally" }))"#,
    );
    assert_eq!(stdout(&flow.run(&[])), "hi wally\n");
}

#[test]
fn args_and_command_are_published() {
    let flow = Flow::new(
        "args",
        r#"hob.term.print(hob.args[2] .. " from " .. hob.command)"#,
    );
    let output = flow.run(&["one", "two"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "two from flow\n");
}

#[test]
fn logs_go_to_stderr() {
    let flow = Flow::new("logs", r#"hob.logs.warn("careful")"#);
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("warn: careful"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn require_serves_only_preloaded_modules() {
    let flow = Flow::new(
        "require",
        r#"
        hob.term.print(tostring(pcall(require, "io")))
        hob.term.print(tostring(require("hob.logs") ~= nil))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "false\ntrue\n");
}
