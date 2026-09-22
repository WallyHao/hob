//! Command discovery through the real binary: precedence, shadowing and the
//! files that are deliberately not commands.

mod common;

use std::fs;

use common::{Sandbox, stderr, stdout};

#[test]
fn project_command_runs_from_a_subdirectory() {
    let sandbox = Sandbox::new("project");
    sandbox.write_project(
        "hello",
        r#"hob.term.print("hello " .. hob.args[1] .. " from " .. hob.command)"#,
    );
    let nested = sandbox.project().join("src").join("deep");
    fs::create_dir_all(&nested).expect("nested dir");

    let output = sandbox.run(&nested, &["hello", "world"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello world from hello\n");
}

#[test]
fn project_shadows_user() {
    let sandbox = Sandbox::new("shadow");
    sandbox.write_project("deploy", r#"hob.term.print("project")"#);
    sandbox.write_user("deploy", r#"hob.term.print("user")"#);

    assert_eq!(stdout(&sandbox.run_project(&["deploy"])), "project\n");

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("project (shadows user)"), "{list}");

    let which = stdout(&sandbox.run_project(&["which", "deploy"]));
    assert!(which.contains("effective  project"), "{which}");
    assert!(which.contains("shadowed   user"), "{which}");

    let removed = sandbox.run_project(&["rm", "deploy"]);
    assert!(removed.status.success(), "{removed:?}");
    let note = stdout(&removed);
    assert!(note.contains("now resolves to"), "{note}");
    assert_eq!(stdout(&sandbox.run_project(&["deploy"])), "user\n");
}

#[test]
fn user_command_runs_outside_a_project() {
    let sandbox = Sandbox::new("user");
    sandbox.write_user("note", r#"hob.term.print("noted")"#);

    let output = sandbox.run(&sandbox.outside(), &["note"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "noted\n");
}

#[test]
fn invalid_and_reserved_files_are_ignored() {
    let sandbox = Sandbox::new("ignored");
    sandbox.write_project("Bad-Name", r#"hob.term.print("nope")"#);
    sandbox.write_project("run", r#"hob.term.print("nope")"#);
    sandbox.write_project("good", r#"hob.term.print("good")"#);

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("name must match [a-z][a-z0-9-]*"), "{list}");
    assert!(list.contains("reserved for the CLI"), "{list}");
    assert!(list.contains("good"), "{list}");

    let unknown = sandbox.run_project(&["Bad-Name"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        stderr(&unknown).contains("unknown command"),
        "{}",
        stderr(&unknown)
    );

    let bare = sandbox.run_project(&["run"]);
    assert_eq!(bare.status.code(), Some(2));
    assert!(
        stderr(&bare).contains("needs a flow file"),
        "{}",
        stderr(&bare)
    );
}

#[test]
fn unknown_command_suggests_the_nearest_name() {
    let sandbox = Sandbox::new("suggest");
    sandbox.write_project("deploy", r#"hob.term.print("deploy")"#);

    let close = sandbox.run_project(&["depliy"]);
    assert_eq!(close.status.code(), Some(2));
    assert!(
        stderr(&close).contains("did you mean `deploy`"),
        "{}",
        stderr(&close)
    );

    let file = sandbox.run_project(&["flow.lua"]);
    assert_eq!(file.status.code(), Some(2));
    assert!(
        stderr(&file).contains("hob run flow.lua"),
        "{}",
        stderr(&file)
    );
}

#[test]
fn summary_comes_from_the_first_doc_line() {
    let sandbox = Sandbox::new("summary");
    sandbox.write_project(
        "greet",
        "--- Say hello to someone.\n\nhob.term.print(\"hi\")",
    );

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("Say hello to someone."), "{list}");
}
