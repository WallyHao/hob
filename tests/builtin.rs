//! The builtin layer: `hob doctor`, and the file layers above it.

mod common;

use std::process::Command;

use common::{Sandbox, stderr, stdout};

#[test]
fn doctor_reports_paths_counts_and_keys() {
    let sandbox = Sandbox::new("doctor");
    sandbox.write_project("hello", r#"hob.term.print("hi")"#);
    sandbox.write_user("note", r#"hob.term.print("hi")"#);
    let output = Command::new(common::BIN)
        .env("HOB_CONFIG_DIR", sandbox.config())
        .env_remove("DEEPSEEK_API_KEY")
        .current_dir(sandbox.project())
        .arg("doctor")
        .output()
        .expect("spawn hob");
    assert!(output.status.success(), "{output:?}");
    let text = stdout(&output);
    assert!(text.contains("config:  "), "{text}");
    assert!(
        text.contains("commands: 3 effective (1 project, 1 user, 1 builtin)"),
        "{text}"
    );
    assert!(text.contains("DEEPSEEK_API_KEY unset"), "{text}");
}

#[test]
fn doctor_never_prints_a_key() {
    let sandbox = Sandbox::new("doctor-key");
    let output = Command::new(common::BIN)
        .env("HOB_CONFIG_DIR", sandbox.config())
        .env("DEEPSEEK_API_KEY", "sk-supersecret")
        .current_dir(sandbox.project())
        .arg("doctor")
        .output()
        .expect("spawn hob");
    let text = stdout(&output);
    assert!(text.contains("DEEPSEEK_API_KEY set"), "{text}");
    assert!(!text.contains("sk-supersecret"), "{text}");
}

#[test]
fn a_file_shadows_a_builtin() {
    let sandbox = Sandbox::new("doctor-shadow");
    sandbox.write_user("doctor", r#"hob.term.print("custom")"#);
    assert_eq!(stdout(&sandbox.run_project(&["doctor"])), "custom\n");

    let which = stdout(&sandbox.run_project(&["which", "doctor"]));
    assert!(which.contains("effective  user"), "{which}");
    assert!(which.contains("shadowed   builtin  compiled in"), "{which}");
}

#[test]
fn a_builtin_cannot_be_removed() {
    let sandbox = Sandbox::new("doctor-rm");
    let removed = sandbox.run_project(&["rm", "doctor"]);
    assert_eq!(removed.status.code(), Some(1));
    assert!(stderr(&removed).contains("builtin"), "{}", stderr(&removed));
}

#[test]
fn builtin_help_comes_from_the_binary() {
    let sandbox = Sandbox::new("doctor-help");
    let output = sandbox.run_project(&["doctor", "--help"]);
    assert!(output.status.success(), "{output:?}");
    let text = stdout(&output);
    assert!(text.contains("doctor -- Report"), "{text}");
    assert!(text.contains("compiled in"), "{text}");
}

#[test]
fn a_builtin_checks_its_argument_count() {
    let sandbox = Sandbox::new("doctor-args");
    let output = sandbox.run_project(&["doctor", "extra"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("usage: doctor"),
        "{}",
        stderr(&output)
    );
}
