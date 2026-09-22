//! `hob new` and `hob rm` through the real binary.

mod common;

use common::{Sandbox, stderr, stdout};

#[test]
fn new_creates_a_runnable_command() {
    let sandbox = Sandbox::new("new");
    let created = sandbox.run_project(&["new", "greet"]);
    assert!(created.status.success(), "{created:?}");
    assert!(
        sandbox.commands().join("greet.lua").exists(),
        "{}",
        stdout(&created)
    );

    let run = sandbox.run_project(&["greet"]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(stdout(&run), "hello from greet\n");

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("TODO: summarize greet."), "{list}");
}

#[test]
fn new_targets_project_or_user() {
    let sandbox = Sandbox::new("targets");
    assert!(sandbox.run_project(&["new", "local"]).status.success());
    assert!(sandbox.commands().join("local.lua").exists());

    let user = sandbox.run_project(&["new", "note", "--user"]);
    assert!(user.status.success(), "{user:?}");
    assert!(sandbox.user_commands().join("note.lua").exists());

    let outside = sandbox.run(&sandbox.outside(), &["new", "global"]);
    assert!(outside.status.success(), "{outside:?}");
    assert!(sandbox.user_commands().join("global.lua").exists());

    let clash = sandbox.run_project(&["new", "twice", "--user", "--local"]);
    assert_eq!(clash.status.code(), Some(2));
}

#[test]
fn new_refuses_duplicates_and_bad_names() {
    let sandbox = Sandbox::new("refuse");
    assert!(sandbox.run_project(&["new", "greet"]).status.success());

    let duplicate = sandbox.run_project(&["new", "greet"]);
    assert_eq!(duplicate.status.code(), Some(1));
    assert!(
        stderr(&duplicate).contains("already exists"),
        "{}",
        stderr(&duplicate)
    );

    let bad = sandbox.run_project(&["new", "Bad_Name"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(
        stderr(&bad).contains("not a valid command name"),
        "{}",
        stderr(&bad)
    );

    let reserved = sandbox.run_project(&["new", "run"]);
    assert_eq!(reserved.status.code(), Some(1));
    assert!(
        stderr(&reserved).contains("reserved"),
        "{}",
        stderr(&reserved)
    );
}

#[test]
fn rm_removes_only_resolved_commands() {
    let sandbox = Sandbox::new("rm");
    sandbox.write_project("gone", r#"hob.term.print("gone")"#);

    let removed = sandbox.run_project(&["rm", "gone"]);
    assert!(removed.status.success(), "{removed:?}");
    assert!(!sandbox.commands().join("gone.lua").exists());

    let again = sandbox.run_project(&["rm", "gone"]);
    assert_eq!(again.status.code(), Some(1));
    assert!(
        stderr(&again).contains("unknown command"),
        "{}",
        stderr(&again)
    );

    let missing = sandbox.run_project(&["rm"]);
    assert_eq!(missing.status.code(), Some(2));
}

#[test]
fn new_and_rm_handle_a_nested_name() {
    let sandbox = Sandbox::new("nested");
    let created = sandbox.run_project(&["new", "tools/hello"]);
    assert!(created.status.success(), "{created:?}");
    assert!(sandbox.commands().join("tools").join("hello.lua").exists());

    let run = sandbox.run_project(&["tools/hello"]);
    assert!(run.status.success(), "{run:?}");

    let removed = sandbox.run_project(&["rm", "tools/hello"]);
    assert!(removed.status.success(), "{removed:?}");
    assert!(!sandbox.commands().join("tools").join("hello.lua").exists());

    let bad = sandbox.run_project(&["new", "tools/Bad"]);
    assert_eq!(bad.status.code(), Some(1));
}
