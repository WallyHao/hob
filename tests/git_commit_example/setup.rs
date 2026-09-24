//! Test repository and model for the git-commit example.
use super::common::{BIN, Sandbox};
use serde_json::json;
use std::fs;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub(super) fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("UTF-8 git output")
}

pub(super) fn setup(tag: &str) -> Sandbox {
    let box_ = Sandbox::new(tag);
    let root = box_.project();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "Hob Test"]);
    git(&root, &["config", "user.email", "hob@example.invalid"]);
    box_.write_project(
        "git-commit",
        include_str!("../../.hob/commands/git-commit.lua"),
    );
    box_.write_project_lib(
        "gitcommit/git",
        include_str!("../../.hob/lib/gitcommit/git.lua"),
    );
    box_.write_project_lib(
        "gitcommit/plan",
        include_str!("../../.hob/lib/gitcommit/plan.lua"),
    );
    fs::write(root.join("a.txt"), "old a\n").expect("seed a");
    fs::write(root.join("b.txt"), "old b\n").expect("seed b");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "seed"]);
    fs::write(root.join("a.txt"), "new a\n").expect("change a");
    fs::write(root.join("b.txt"), "new b\n").expect("change b");
    box_
}

pub(super) fn run(box_: &Sandbox, input: &str) -> Output {
    let mut child = Command::new(BIN)
        .current_dir(box_.project())
        .env("HOB_CONFIG_DIR", box_.config())
        .env("TEST_KEY", "test-key")
        .arg("git-commit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hob");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("feed answers");
    child.wait_with_output().expect("hob exits")
}

pub(super) async fn provider(box_: &Sandbox) -> MockServer {
    let server = MockServer::start().await;
    let response = json!({ "groups": [
        { "title": "feat: update a", "body": "", "why": "a changes", "ids": [1] },
        { "title": "feat: update b", "body": "", "why": "b changes", "ids": [2] }
    ], "skip": [] });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{ "message": { "role": "assistant", "content": response.to_string() } }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 10, "total_tokens": 20 }
        })))
        .mount(&server)
        .await;
    fs::write(box_.config().join("config.toml"), format!(
        "[defaults]\nprovider = \"local\"\nmodel = \"m\"\n[providers.local]\nbase_url = \"{}\"\napi_key_env = \"TEST_KEY\"\n", server.uri()
    )).expect("provider config");
    server
}
