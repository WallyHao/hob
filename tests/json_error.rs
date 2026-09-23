//! `--json` failures: one object on stderr, once the command is known.

mod common;

use common::{Sandbox, stderr, stdout};
use serde_json::Value;

/// The one object an error prints on stderr.
fn json_err(output: &std::process::Output) -> Value {
    serde_json::from_str(&stderr(output)).expect("one JSON error object")
}

#[test]
fn an_error_is_one_json_object_on_stderr() {
    let sandbox = Sandbox::new("json-error");
    let output = sandbox.run_project(&["which", "nowhere", "--json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stdout(&output).is_empty(), "{output:?}");
    assert!(
        json_err(&output)["error"]
            .as_str()
            .is_some_and(|message| message.contains("nowhere")),
        "{output:?}"
    );
}

#[test]
fn a_usage_error_after_the_command_is_json_too() {
    let sandbox = Sandbox::new("json-usage");
    let output = sandbox.run_project(&["list", "extra", "--json"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        json_err(&output)["error"]
            .as_str()
            .is_some_and(|message| message.contains("no arguments")),
        "{output:?}"
    );
}
