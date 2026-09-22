//! The remaining control flags: verbosity, `--yes`, conflicts and `--`.

mod common;

use std::process::Command;

use common::{Flow, stderr, stdout};

#[test]
fn yes_answers_questions_with_defaults() {
    let flow = Flow::new(
        "yes",
        r#"
        hob.term.print(hob.term.input{ prompt = "name? ", default = "anon" })
        hob.term.print(tostring(hob.term.allow("go?", { default = true })))
        local picked = hob.term.select{ prompt = "which?", options = { "a", "b" }, default = "b" }
        hob.term.print(tostring(picked))
        "#,
    );
    let output = flow.run(&["--yes"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "anon\ntrue\nb\n");
}

#[test]
fn quiet_and_verbose_control_log_lines() {
    let flow = Flow::new(
        "logs",
        r#"
        hob.logs.debug("detail")
        hob.logs.info("progress")
        hob.logs.warn("careful")
        "#,
    );
    let normal = flow.run(&[]);
    let err = stderr(&normal);
    assert!(err.contains("progress") && err.contains("careful"), "{err}");
    assert!(!err.contains("detail"), "{err}");

    let verbose = flow.run(&["-vv"]);
    assert!(stderr(&verbose).contains("detail"), "{verbose:?}");

    let quiet = flow.run(&["-q"]);
    let err = stderr(&quiet);
    assert!(err.contains("careful"), "{err}");
    assert!(!err.contains("progress"), "{err}");
}

#[test]
fn dry_run_and_step_cannot_be_combined() {
    let flow = Flow::new("conflict", "");
    let output = flow.run(&["--dry-run", "--step"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("conflicts with"), "{output:?}");
}

#[test]
fn double_dash_sends_flags_to_the_flow() {
    let flow = Flow::new("dashdash", r"hob.term.print(hob.args[1])");
    let output = flow.run_with(&["--", "--dry-run"], &[], None);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "--dry-run\n");
}

#[test]
fn an_unknown_flag_before_the_command_is_refused() {
    let output = Command::new(common::BIN)
        .arg("--bogus")
        .output()
        .expect("spawn hob");
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unknown argument"), "{output:?}");
}
