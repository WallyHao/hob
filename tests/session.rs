//! `hob.proc` sessions through the real binary: environment, directory,
//! profile and lifecycle.

mod common;

use common::{BIN, Flow, stderr, stdout};

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

/// Run `hob doctor` from inside a session, so the key report comes from a real
/// child process with the environment the session gave it.
fn probe(tag: &str, env_clear: bool) -> Flow {
    Flow::new(
        tag,
        &format!(
            r#"
            local session = hob.proc.open{{ env_clear = {env_clear} }}
            local result = session:exec({{ hob.args[1], "doctor" }}, {{ trim = true }})
            hob.term.print(result.stdout:match("DEEPSEEK_API_KEY (%w+)"))
            "#
        ),
    )
}

#[test]
fn env_clear_hides_the_process_environment() {
    let env = [("DEEPSEEK_API_KEY", "sk-test")];

    let inherited = probe("session-inherit", false).run_with(&[BIN], &env, None);
    assert!(inherited.status.success(), "{inherited:?}");
    assert_eq!(stdout(&inherited), "set\n");

    let cleared = probe("session-clear", true).run_with(&[BIN], &env, None);
    assert!(cleared.status.success(), "{cleared:?}");
    assert_eq!(stdout(&cleared), "unset\n");
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
