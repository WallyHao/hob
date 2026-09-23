// --- library links ---
// Code provenance must survive file, directory and root symlinks.

#![cfg(unix)]

mod common;

use std::fs;
use std::os::unix::fs::symlink;

use common::{Sandbox, stderr, stdout};

const FLOW: &str = r#"hob.term.print(require("greet").who)"#;
const SOURCE: &str = r#"return { who = "linked" }"#;

fn denied(sandbox: &Sandbox) {
    let output = sandbox.run_project(&["hello"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stdout(&output).is_empty(), "{output:?}");
    assert!(
        stderr(&output).contains("outside authorized directory"),
        "{output:?}"
    );
}

#[test]
fn a_file_link_cannot_escape_or_fall_back_to_the_user_library() {
    let sandbox = Sandbox::new("lib-link-file");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", SOURCE);
    let root = sandbox.project().join(".hob/lib");
    let sibling = sandbox.project().join(".hob/lib-other");
    fs::create_dir_all(&root).expect("library directory");
    fs::create_dir_all(&sibling).expect("sibling directory");
    fs::write(sibling.join("greet.lua"), SOURCE).expect("outside module");
    symlink(sibling.join("greet.lua"), root.join("greet.lua")).expect("file link");
    denied(&sandbox);
}

#[test]
fn a_directory_link_cannot_escape_the_library() {
    let sandbox = Sandbox::new("lib-link-directory");
    sandbox.write_user("hello", r#"hob.term.print(require("util.greet").who)"#);
    let root = sandbox.project().join(".hob/lib");
    fs::create_dir_all(&root).expect("library directory");
    fs::write(sandbox.outside().join("greet.lua"), SOURCE).expect("outside module");
    symlink(sandbox.outside(), root.join("util")).expect("directory link");
    denied(&sandbox);
}

#[test]
fn library_roots_cannot_escape_their_owner() {
    for layer in ["project", "user", "hob"] {
        let sandbox = Sandbox::new(&format!("lib-link-root-{layer}"));
        sandbox.write_user("hello", FLOW);
        fs::write(sandbox.outside().join("greet.lua"), SOURCE).expect("outside module");
        let link = match layer {
            "user" => sandbox.config().join("lib"),
            "hob" => sandbox.project().join(".hob"),
            _ => sandbox.project().join(".hob/lib"),
        };
        if layer == "hob" {
            fs::create_dir(sandbox.outside().join("lib")).expect("outside library");
        }
        fs::create_dir_all(link.parent().expect("parent")).expect("link parent");
        symlink(sandbox.outside(), link).expect("root link");
        denied(&sandbox);
    }
}

#[test]
fn internal_file_and_directory_links_work() {
    let sandbox = Sandbox::new("lib-link-internal");
    sandbox.write_user("hello", r#"hob.term.print(require("alias.greet").who)"#);
    sandbox.write_project_lib("util/original", SOURCE);
    let root = sandbox.project().join(".hob/lib");
    symlink("original.lua", root.join("util/greet.lua")).expect("file link");
    symlink("util", root.join("alias")).expect("directory link");
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "linked\n");
}

#[test]
fn a_library_root_may_link_within_its_owner() {
    let sandbox = Sandbox::new("lib-link-internal-root");
    sandbox.write_user("hello", FLOW);
    let helpers = sandbox.project().join("helpers");
    fs::create_dir(&helpers).expect("helpers directory");
    fs::write(helpers.join("greet.lua"), SOURCE).expect("helper module");
    fs::create_dir(sandbox.project().join(".hob")).expect("hob directory");
    symlink("../helpers", sandbox.project().join(".hob/lib")).expect("root link");
    let output = sandbox.run_project(&["hello"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "linked\n");
}

#[test]
fn a_dangling_file_link_is_an_error_not_a_fallback() {
    let sandbox = Sandbox::new("lib-link-dangling");
    sandbox.write_user("hello", FLOW);
    sandbox.write_user_lib("greet", SOURCE);
    let root = sandbox.project().join(".hob/lib");
    fs::create_dir_all(&root).expect("library directory");
    symlink("missing.lua", root.join("greet.lua")).expect("dangling link");
    let output = sandbox.run_project(&["hello"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("cannot resolve library path"),
        "{output:?}"
    );
}
