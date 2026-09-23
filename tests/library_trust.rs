// --- library trust ---
// Trusted entry points must not import code supplied by an untrusted checkout.

mod common;

use common::{Sandbox, stderr, stdout};

const FLOW: &str = r#"hob.term.print(require("greet").who)"#;
const USER: &str = r#"return { who = "user" }"#;
const PROJECT: &str = r#"return { who = "project" }"#;

#[test]
fn an_untrusted_project_cannot_shadow_a_user_library() {
    let sandbox = Sandbox::untrusted("lib-user-trust");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", USER);
    sandbox.write_project_lib("greet", r#"error("untrusted code executed")"#);
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}

#[test]
fn a_project_only_library_is_unavailable_without_trust() {
    let sandbox = Sandbox::untrusted("lib-only-trust");
    sandbox.write_user("hello", FLOW);
    sandbox.write_project_lib("greet", PROJECT);
    let output = sandbox.run_project(&["hello"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("no library `greet`"), "{output:?}");
    assert!(stdout(&output).is_empty());
}

#[test]
fn explicit_execution_does_not_authorize_project_libraries() {
    let sandbox = Sandbox::untrusted("lib-explicit-trust");
    sandbox.write_user_lib("greet", USER);
    sandbox.write_project_lib("greet", PROJECT);
    std::fs::write(sandbox.project().join("flow.lua"), FLOW).expect("flow file");
    let output = sandbox.run_project(&["run", "flow.lua"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
    sandbox.trust();
    let trusted = sandbox.run_project(&["run", "flow.lua"]);
    assert!(trusted.status.success(), "{trusted:?}");
    assert_eq!(stdout(&trusted), "project\n");
}

#[test]
fn granting_and_revoking_trust_changes_library_precedence() {
    let sandbox = Sandbox::untrusted("lib-revoke-trust");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", USER);
    sandbox.write_project_lib("greet", PROJECT);
    sandbox.trust();
    let trusted = sandbox.run_project(&["hello"]);
    assert!(trusted.status.success(), "{trusted:?}");
    assert_eq!(stdout(&trusted), "project\n");
    let revoked = sandbox.run_project(&["trust", "--revoke"]);
    assert!(revoked.status.success(), "{revoked:?}");
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}

#[test]
fn a_corrupt_trust_store_fails_before_the_flow_runs() {
    let sandbox = Sandbox::untrusted("lib-corrupt-trust");
    sandbox.write_user("hello", r#"hob.term.print("entry executed")"#);
    sandbox.write_project_lib("greet", PROJECT);
    std::fs::write(sandbox.config().join("trust.json"), "invalid").expect("trust file");
    let output = sandbox.run_project(&["hello"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("cannot parse"), "{output:?}");
    assert!(stderr(&output).contains("trust.json"), "{output:?}");
}

#[test]
fn a_user_library_works_outside_a_project() {
    let sandbox = Sandbox::untrusted("lib-outside-trust");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", USER);
    let output = sandbox.run(&sandbox.outside(), &["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}

#[test]
fn library_authorization_is_fixed_before_the_flow_starts() {
    let sandbox = Sandbox::untrusted("lib-snapshot-trust");
    sandbox.write_user_lib("greet", USER);
    sandbox.write_project_lib("greet", PROJECT);
    sandbox.write_user(
        "hello",
        r#"
        hob.file.write(hob.args[1], hob.json.encode({ projects = { hob.args[2] } }))
        hob.term.print(require("greet").who)
        "#,
    );
    let trust = sandbox.config().join("trust.json");
    let root = sandbox.project().canonicalize().expect("canonical root");
    let output = sandbox.run_project(&[
        "hello",
        trust.to_str().expect("trust path"),
        root.to_str().expect("project path"),
    ]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}
