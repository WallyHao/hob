//! `term.print` styling: escapes only when the run asked for them.

mod common;

use common::{Flow, stdout};

const STYLED: &str = r#"hob.term.print("hi", { style = "error" })"#;

#[test]
fn a_pipe_gets_plain_text() {
    let output = Flow::new("plain", STYLED).run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hi\n");
}

#[test]
fn always_colours_a_pipe() {
    let output = Flow::new("always", STYLED).run(&["--color=always"]);
    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout(&output).contains("\u{1b}[31mhi\u{1b}[0m"),
        "{output:?}"
    );
}

#[test]
fn never_beats_the_force_variable() {
    let output = Flow::new("never", STYLED).run_with(
        &["--color", "never"],
        &[("CLICOLOR_FORCE", "1")],
        None,
    );
    assert_eq!(stdout(&output), "hi\n");
}

#[test]
fn always_beats_no_color() {
    let output =
        Flow::new("force", STYLED).run_with(&["--color", "always"], &[("NO_COLOR", "1")], None);
    assert!(stdout(&output).contains("\u{1b}[31m"), "{output:?}");
}

#[test]
fn no_color_keeps_a_pipe_plain() {
    let output = Flow::new("no-color", STYLED).run_with(&[], &[("NO_COLOR", "1")], None);
    assert_eq!(stdout(&output), "hi\n");
}

#[test]
fn a_bad_mode_is_a_usage_error() {
    let output = Flow::new("bad-mode", STYLED).run(&["--color=sometimes"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
}
