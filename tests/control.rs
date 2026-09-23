//! Previewing and approving effects: `--dry-run` and `--step`.

mod common;

use common::{Flow, stderr, stdout};

#[test]
fn dry_run_performs_reads_and_refuses_writes() {
    let flow = Flow::new(
        "dry-read",
        r#"
        local text = hob.file.read("input.txt")
        hob.term.print(text)
        hob.file.write("output.txt", "written")
        "#,
    );
    flow.write("input.txt", "hello");
    let output = flow.run(&["--dry-run"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello\n");
    assert!(!flow.dir().join("output.txt").exists());
    assert!(
        stderr(&output).contains("dry-run: file.write"),
        "{output:?}"
    );
}

#[test]
fn dry_run_refuses_a_command() {
    let flow = Flow::new(
        "dry-exec",
        r#"
        local result = hob.proc.exec{ "touch", "ran.txt" }
        hob.term.print("code " .. result.code)
        "#,
    );
    let output = flow.run(&["--dry-run"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "code 0\n");
    assert!(!flow.dir().join("ran.txt").exists());
    assert!(
        stderr(&output).contains("dry-run: proc.exec touch ran.txt"),
        "{output:?}"
    );
}

#[test]
fn dry_run_answers_a_schema_call_with_an_example() {
    let flow = Flow::new(
        "dry-ask",
        r#"
        local answer, meta = hob.agent.ask{
          provider = "deepseek",
          model = "deepseek-chat",
          prompt = "hi",
          schema = {
            type = "object",
            required = { "title" },
            properties = { title = { type = "string" } },
          },
        }
        hob.term.print(answer.title .. " " .. meta.attempts)
        "#,
    );
    let output = flow.run(&["--dry-run"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "mock 0\n");
    assert!(stderr(&output).contains("dry-run: agent.ask"), "{output:?}");
}

#[test]
fn step_runs_what_is_approved() {
    let flow = Flow::new(
        "step",
        r#"
        hob.file.write("first.txt", "a")
        hob.file.write("second.txt", "b")
        "#,
    );
    let output = flow.run_with(&["--step"], &[], Some("y\nn\n"));
    assert!(output.status.success(), "{output:?}");
    assert!(flow.dir().join("first.txt").exists());
    assert!(!flow.dir().join("second.txt").exists());
    assert!(
        stderr(&output).contains("declined: file.write"),
        "{output:?}"
    );
}

#[test]
fn step_masks_a_secret_from_the_environment() {
    let flow = Flow::new("step-secret", r#"hob.proc.shell("echo " .. hob.args[1])"#);
    let output = flow.run_with(
        &["--step", "supersecret123"],
        &[("TEST_API_KEY", "supersecret123")],
        Some("n\n"),
    );
    assert!(output.status.success(), "{output:?}");
    let err = stderr(&output);
    assert!(err.contains("***"), "{err}");
    assert!(!err.contains("supersecret123"), "{err}");
}

#[test]
fn dry_run_refuses_a_model_listing() {
    let flow = Flow::new(
        "dry-list",
        r#"
        local provider = { id = "test", base_url = "http://127.0.0.1:1", api_key_env = "NO_KEY" }
        local models = hob.agent.list{ provider = provider }
        hob.term.print(tostring(#models))
        "#,
    );
    let output = flow.run(&["--dry-run"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "0\n");
    assert!(
        stderr(&output).contains("dry-run: agent.list"),
        "{output:?}"
    );
}

#[test]
fn step_without_stdin_is_an_error() {
    let flow = Flow::new("step-closed", r#"hob.file.write("x.txt", "y")"#);
    let output = flow.run(&["--step"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("input closed"), "{output:?}");
}
