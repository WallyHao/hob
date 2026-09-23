//! `hob.proc` one-shot commands through the real binary.

mod common;

use common::{Flow, stdout};

#[test]
fn exec_captures_output_and_code() {
    let flow = Flow::new(
        "exec",
        r#"
        local result = hob.proc.exec({ "sh", "-c", "echo out; echo err >&2; exit 3" })
        hob.term.print(result.code .. " " .. result.stdout:gsub("\n", "") .. " " .. result.stderr:gsub("\n", "") .. " " .. tostring(result.ok))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "3 out err false\n");
}

#[test]
fn shell_runs_a_line() {
    let flow = Flow::new(
        "shell",
        r#"hob.term.print(hob.proc.shell("printf a; printf b", { trim = true }).stdout)"#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "ab\n");
}

#[test]
fn stdin_reaches_the_child() {
    let flow = Flow::new(
        "stdin",
        r#"hob.term.print(hob.proc.exec({ "cat" }, { stdin = "hello\n", trim = true }).stdout)"#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello\n");
}

#[test]
fn a_large_stdin_and_a_large_output_do_not_deadlock() {
    // The child fills stdout before it reads a byte of stdin, so a parent that
    // wrote stdin before draining the pipes would block forever.
    let flow = Flow::new(
        "pipe-load",
        r#"
        local text = string.rep("x", 200000)
        local result = hob.proc.shell("head -c 200000 /dev/zero; cat", { stdin = text, timeout_ms = 20000 })
        hob.assert(result.ok, "child failed: " .. result.stderr)
        hob.term.print(tostring(#result.stdout))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "400000\n");
}

#[test]
fn which_finds_programs_and_reports_the_rest_as_nil() {
    let flow = Flow::new(
        "which",
        r#"
        hob.term.print(tostring(hob.proc.which("sh") ~= nil))
        hob.term.print(tostring(hob.proc.which("definitely-not-a-program")))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "true\nnil\n");
}

#[test]
fn a_timeout_kills_the_child_and_reports_124() {
    let flow = Flow::new(
        "timeout",
        r#"
        local result = hob.proc.exec({ "sleep", "5" }, { timeout_ms = 100 })
        hob.term.print(result.code .. " " .. tostring(result.ok))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "124 false\n");
}

#[test]
fn inherit_hands_the_terminal_over() {
    let flow = Flow::new(
        "inherit",
        r#"
        local result = hob.proc.exec({ "sh", "-c", "echo inherited" }, { inherit = true })
        hob.term.print(tostring(result.stdout == "" and result.ok))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "inherited\ntrue\n");
}
