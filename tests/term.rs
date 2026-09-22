//! `hob.term` questions through the real binary, with piped answers.

mod common;

use common::{Flow, stdout};

#[test]
fn input_reads_a_line_or_takes_the_default() {
    let flow = Flow::new(
        "input",
        r#"
        hob.term.print(hob.term.input{ prompt = "name? ", default = "anon" })
        hob.term.print(hob.term.input{ prompt = "name? " })
        "#,
    );
    let output = flow.run_with(&[], &[], Some("\nwally\n"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "anon\nwally\n");
}

#[test]
fn allow_answers_yes_no() {
    let flow = Flow::new(
        "allow",
        r#"
        hob.term.print(tostring(hob.term.allow("go?", { default = false })))
        hob.term.print(tostring(hob.term.allow("go?")))
        "#,
    );
    let output = flow.run_with(&[], &[], Some("\nyes\n"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "false\ntrue\n");
}

#[test]
fn select_returns_the_value_of_the_chosen_label() {
    let flow = Flow::new(
        "select",
        r#"
        local picked = hob.term.select{
          prompt = "which?",
          options = { "one", { label = "two", value = 2 } },
          default = "one",
        }
        hob.term.print(tostring(picked))
        "#,
    );
    let output = flow.run_with(&[], &[], Some("2\n"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "2\n");
}

#[test]
fn choose_returns_several_picks() {
    let flow = Flow::new(
        "choose",
        r#"
        local picked = hob.term.choose{
          prompt = "which?",
          options = { "a", "b", "c" },
          min = 1,
        }
        hob.term.print(table.concat(picked, ","))
        "#,
    );
    let output = flow.run_with(&[], &[], Some("1, c\n"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "a,c\n");
}

#[test]
fn a_closed_stdin_fails_instead_of_answering() {
    let flow = Flow::new("closed", r#"hob.term.input{ prompt = "name? " }"#);
    let output = flow.run_with(&[], &[], Some(""));
    assert_eq!(output.status.code(), Some(1));
    assert!(
        common::stderr(&output).contains("input closed"),
        "{}",
        common::stderr(&output)
    );
}
