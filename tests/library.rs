//! Shared Lua libraries: `require` resolves the project and user `lib/` trees.

mod common;

use common::{Sandbox, stderr, stdout};

#[test]
fn a_command_can_require_a_project_library() {
    let sandbox = Sandbox::new("lib-project");
    sandbox.write_project_lib(
        "greet",
        r#"return { hello = function(name) return "hi " .. name end }"#,
    );
    sandbox.write_project(
        "hello",
        r#"
        local greet = require("greet")
        hob.term.print(greet.hello("world"))
        "#,
    );
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hi world\n");
}

#[test]
fn a_nested_module_name_maps_to_a_subdirectory() {
    let sandbox = Sandbox::new("lib-nested");
    sandbox.write_project_lib(
        "util/text",
        r"return { shout = function(text) return text:upper() end }",
    );
    sandbox.write_project(
        "shout",
        r#"hob.term.print(require("util.text").shout("hey"))"#,
    );
    let output = sandbox.run_project(&["shout"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "HEY\n");
}

#[test]
fn a_project_library_wins_over_the_user_library() {
    let sandbox = Sandbox::new("lib-shadow");
    sandbox.write_project_lib("greet", r#"return { who = "project" }"#);
    sandbox.write_user_lib("greet", r#"return { who = "user" }"#);
    sandbox.write_project("who", r#"hob.term.print(require("greet").who)"#);
    let output = sandbox.run_project(&["who"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "project\n");
}

#[test]
fn a_missing_library_names_itself() {
    let sandbox = Sandbox::new("lib-missing");
    sandbox.write_project("missing", r#"require("nope")"#);
    let output = sandbox.run_project(&["missing"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no library `nope`"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_path_like_name_is_refused() {
    let sandbox = Sandbox::new("lib-path");
    sandbox.write_project_lib("util/text", r#"return { who = "loaded" }"#);
    sandbox.write_project("slash", r#"require("util/text")"#);
    let output = sandbox.run_project(&["slash"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no library"),
        "{}",
        stderr(&output)
    );
}
