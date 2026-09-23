// --- library roots ---
// User libraries obey the same containment rule, while untrusted roots are
// ignored before inspecting their links.

#![cfg(unix)]

mod common;

use std::fs;
use std::os::unix::fs::symlink;

use common::{Sandbox, stderr, stdout};

const FLOW: &str = r#"hob.term.print(require("greet").who)"#;
const SOURCE: &str = r#"return { who = "user" }"#;

#[test]
fn a_user_library_cannot_link_to_untrusted_project_code() {
    let sandbox = Sandbox::untrusted("lib-user-link-project");
    sandbox.write_user("hello", FLOW);
    sandbox.write_project_lib("greet", SOURCE);
    let root = sandbox.config().join("lib");
    fs::create_dir(&root).expect("user library");
    symlink(
        sandbox.project().join(".hob/lib/greet.lua"),
        root.join("greet.lua"),
    )
    .expect("module link");
    let output = sandbox.run_project(&["hello"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("outside authorized directory"),
        "{output:?}"
    );
}

#[test]
fn an_untrusted_root_link_does_not_block_a_user_library() {
    let sandbox = Sandbox::untrusted("lib-ignore-root-link");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", SOURCE);
    fs::create_dir(sandbox.project().join(".hob")).expect("hob directory");
    symlink(sandbox.outside(), sandbox.project().join(".hob/lib")).expect("root link");
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}

#[test]
fn a_missing_project_library_still_falls_back_to_the_user() {
    let sandbox = Sandbox::new("lib-missing-fallback");
    sandbox.write_user("hello", FLOW);
    sandbox.write_project_lib("other", SOURCE);
    sandbox.write_user_lib("greet", SOURCE);
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user\n");
}
