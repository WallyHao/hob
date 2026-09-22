//! Subdirectories as namespaces: `foo/bar.lua` is the command `foo/bar`.

mod common;

use common::{Sandbox, stdout};

#[test]
fn a_nested_command_runs_and_lists() {
    let sandbox = Sandbox::new("nested");
    sandbox.write_project("git/commit", r#"hob.term.print("commit " .. hob.command)"#);
    sandbox.write_project("git/pr/close", r#"hob.term.print("close")"#);

    let run = sandbox.run_project(&["git/commit"]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(stdout(&run), "commit git/commit\n");

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("git/commit"), "{list}");
    assert!(list.contains("git/pr/close"), "{list}");
}

#[test]
fn a_nested_name_shadows_across_layers() {
    let sandbox = Sandbox::new("nested-shadow");
    sandbox.write_project("tools/deploy", r#"hob.term.print("project")"#);
    sandbox.write_user("tools/deploy", r#"hob.term.print("user")"#);

    assert_eq!(stdout(&sandbox.run_project(&["tools/deploy"])), "project\n");
    let which = stdout(&sandbox.run_project(&["which", "tools/deploy"]));
    assert!(which.contains("shadowed"), "{which}");
}

#[test]
fn a_bad_segment_is_ignored() {
    let sandbox = Sandbox::new("nested-bad");
    sandbox.write_project("Good/thing", r#"hob.term.print("nope")"#);
    sandbox.write_project("ok/thing", r#"hob.term.print("ok")"#);

    let list = stdout(&sandbox.run_project(&["list"]));
    assert!(list.contains("ignored"), "{list}");
    assert!(list.contains("ok/thing"), "{list}");

    let unknown = sandbox.run_project(&["Good/thing"]);
    assert_eq!(unknown.status.code(), Some(2));
}
