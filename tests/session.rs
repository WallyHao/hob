//! `hob.proc` sessions through the real binary: environment, directory,
//! profile and lifecycle.

mod common;

use common::{Flow, stderr, stdout};

#[test]
fn a_session_keeps_environment_and_directory() {
    let flow = Flow::new(
        "session",
        r#"
        local session = hob.proc.open{ env = { GREETING = "hi" } }
        hob.term.print(session:exec({ "sh", "-c", "echo $GREETING" }, { trim = true }).stdout)
        session:setenv("GREETING", "bye")
        hob.term.print(session:exec({ "sh", "-c", "echo $GREETING" }, { trim = true }).stdout)
        session:unset("GREETING")
        hob.term.print(session:exec({ "sh", "-c", "echo ${GREETING:-unset}" }, { trim = true }).stdout)
        session:chdir("sub")
        hob.term.print(session:exec({ "pwd" }, { trim = true }).stdout)
        session:reset()
        hob.term.print(session:exec({ "pwd" }, { trim = true }).stdout)
        session:close()
        "#,
    );
    flow.write("sub/keep.txt", "");
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    let dir = flow.dir().display().to_string();
    assert_eq!(
        stdout(&output),
        format!("hi\nbye\nunset\n{dir}/sub\n{dir}\n")
    );
}

#[test]
fn the_profile_runs_before_each_shell_line() {
    let flow = Flow::new(
        "profile",
        r#"
        local session = hob.proc.open{ profile = "export FROM_PROFILE=yes" }
        hob.term.print(session:shell("echo $FROM_PROFILE", { trim = true }).stdout)
        session:setup("export FROM_PROFILE=changed")
        hob.term.print(session:shell("echo $FROM_PROFILE", { trim = true }).stdout)
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "yes\nchanged\n");
}

#[test]
fn a_closed_session_is_an_error() {
    let flow = Flow::new(
        "closed",
        r"
        local session = hob.proc.open{}
        session:close()
        session:state()
        ",
    );
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no session with handle 1"),
        "{}",
        stderr(&output)
    );
}
