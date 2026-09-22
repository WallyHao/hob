//! Per-command `--help` and the `args:` header check.

mod common;

use common::{Sandbox, stderr, stdout};

#[test]
fn help_prints_the_header_without_running_the_flow() {
    let sandbox = Sandbox::new("help");
    sandbox.write_project(
        "hello",
        r#"
        --- Say hello to someone.
        --- usage: hello <name> [--loud]
        --- args: 1..2

        hob.term.print("ran")
        "#,
    );
    let output = sandbox.run_project(&["hello", "--help"]);
    assert!(output.status.success(), "{output:?}");
    let text = stdout(&output);
    assert!(text.contains("hello -- Say hello to someone."), "{text}");
    assert!(text.contains("usage: hello <name> [--loud]"), "{text}");
    assert!(text.contains("1 to 2 arguments"), "{text}");
    assert!(!text.contains("ran"), "{text}");
}

#[test]
fn a_wrong_argument_count_exits_two_with_the_usage() {
    let sandbox = Sandbox::new("args");
    sandbox.write_project(
        "hello",
        r#"
        --- Say hello.
        --- usage: hello <name>
        --- args: 1

        hob.term.print("hi " .. hob.args[1])
        "#,
    );
    let missing = sandbox.run_project(&["hello"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        stderr(&missing).contains("usage: hello <name>"),
        "{}",
        stderr(&missing)
    );

    let extra = sandbox.run_project(&["hello", "a", "b"]);
    assert_eq!(extra.status.code(), Some(2));

    let ok = sandbox.run_project(&["hello", "world"]);
    assert!(ok.status.success(), "{ok:?}");
    assert_eq!(stdout(&ok), "hi world\n");
}

#[test]
fn the_range_forms_are_understood() {
    let sandbox = Sandbox::new("ranges");
    sandbox.write_project(
        "many",
        "--- args: 2+\n\nhob.term.print(tostring(#hob.args))",
    );
    sandbox.write_project(
        "few",
        "--- args: 0..2\n\nhob.term.print(tostring(#hob.args))",
    );

    assert_eq!(
        stdout(&sandbox.run_project(&["many", "a", "b", "c"])),
        "3\n"
    );
    assert_eq!(sandbox.run_project(&["many", "a"]).status.code(), Some(2));
    assert_eq!(stdout(&sandbox.run_project(&["few"])), "0\n");
    assert_eq!(
        sandbox.run_project(&["few", "a", "b", "c"]).status.code(),
        Some(2)
    );
}

#[test]
fn an_unparseable_args_line_is_ignored() {
    let sandbox = Sandbox::new("unparseable");
    sandbox.write_project(
        "loose",
        "--- args: many\n\nhob.term.print(tostring(#hob.args))",
    );
    let output = sandbox.run_project(&["loose", "a", "b", "c"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "3\n");
}
