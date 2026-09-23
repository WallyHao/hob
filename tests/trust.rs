//! Project trust: a project's commands run only after `hob trust`.

mod common;

use common::{Sandbox, stderr, stdout};

#[test]
fn a_project_command_is_refused_until_the_project_is_trusted() {
    let sandbox = Sandbox::untrusted("denied");
    sandbox.write_project("hello", r#"hob.term.print("ran")"#);

    let denied = sandbox.run_project(&["hello"]);
    assert_eq!(denied.status.code(), Some(1));
    assert!(stdout(&denied).is_empty());
    assert!(stderr(&denied).contains("not trusted"), "{denied:?}");

    let granted = sandbox.run_project(&["trust"]);
    assert!(granted.status.success(), "{granted:?}");
    assert!(stdout(&granted).contains("trusted"), "{granted:?}");

    let allowed = sandbox.run_project(&["hello"]);
    assert!(allowed.status.success(), "{allowed:?}");
    assert_eq!(stdout(&allowed), "ran\n");
}

#[test]
fn revoking_takes_the_command_away_again() {
    let sandbox = Sandbox::new("revoke");
    sandbox.write_project("hello", r#"hob.term.print("ran")"#);
    assert!(sandbox.run_project(&["hello"]).status.success());

    let revoked = sandbox.run_project(&["trust", "--revoke"]);
    assert!(revoked.status.success(), "{revoked:?}");
    assert_eq!(sandbox.run_project(&["hello"]).status.code(), Some(1));
}

#[test]
fn a_refusal_does_not_fall_back_to_a_shadowed_user_command() {
    let sandbox = Sandbox::untrusted("shadow");
    sandbox.write_project("hello", r#"hob.term.print("project")"#);
    sandbox.write_user("hello", r#"hob.term.print("user")"#);

    let denied = sandbox.run_project(&["hello"]);
    assert_eq!(denied.status.code(), Some(1));
    assert!(stdout(&denied).is_empty());
}

#[test]
fn listing_marks_an_untrusted_project_command() {
    let sandbox = Sandbox::untrusted("listed");
    sandbox.write_project("hello", "");
    let listed = stdout(&sandbox.run_project(&["list"]));
    assert!(listed.contains("untrusted"), "{listed}");
}

#[test]
fn explicit_flows_and_user_commands_do_not_need_trust() {
    let sandbox = Sandbox::untrusted("explicit");
    sandbox.write_user("hello", r#"hob.term.print("user")"#);
    assert_eq!(
        stdout(&sandbox.run(&sandbox.outside(), &["hello"])),
        "user\n"
    );

    std::fs::write(
        sandbox.project().join("flow.lua"),
        r#"hob.term.print("flow")"#,
    )
    .expect("flow file");
    assert_eq!(stdout(&sandbox.run_project(&["run", "flow.lua"])), "flow\n");
}

#[test]
fn a_project_cannot_trust_itself() {
    let sandbox = Sandbox::untrusted("self");
    sandbox.write_project("hello", r#"hob.term.print("ran")"#);
    std::fs::write(
        sandbox.project().join(".hob").join("trust.json"),
        r#"{"projects": ["/"]}"#,
    )
    .expect("planted trust file");
    assert_eq!(sandbox.run_project(&["hello"]).status.code(), Some(1));
}

#[test]
fn help_is_available_for_an_untrusted_command() {
    let sandbox = Sandbox::untrusted("help");
    sandbox.write_project("hello", "--- Say hello.\n");
    let output = sandbox.run_project(&["hello", "--help"]);
    assert!(output.status.success(), "{output:?}");
    assert!(stdout(&output).contains("Say hello"), "{output:?}");
}

#[test]
fn trust_list_shows_the_trusted_root() {
    let sandbox = Sandbox::untrusted("inspect");
    let empty = stdout(&sandbox.run_project(&["trust", "--list"]));
    assert!(empty.contains("no trusted projects"), "{empty}");

    sandbox.trust();
    let listed = stdout(&sandbox.run_project(&["trust", "--list"]));
    let root = sandbox.project().canonicalize().expect("canonical root");
    assert!(listed.contains(&root.display().to_string()), "{listed}");
}
