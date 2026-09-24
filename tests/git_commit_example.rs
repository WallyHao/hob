//! The bundled git-commit command against a real repository and mock model.
mod common;
#[path = "git_commit_example/setup.rs"]
mod setup;

use common::{stderr, stdout};
use setup::{git, provider, run, setup};

#[test]
fn declining_share_leaves_git_untouched() {
    let box_ = setup("git-commit-cancel");
    let before = git(&box_.project(), &["rev-parse", "HEAD"]);
    let out = run(&box_, "n\n");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(git(&box_.project(), &["rev-parse", "HEAD"]), before);
    assert!(git(&box_.project(), &["diff", "--cached", "--name-only"]).is_empty());
}

#[test]
fn existing_staged_changes_are_refused() {
    let box_ = setup("git-commit-staged");
    git(&box_.project(), &["add", "a.txt"]);
    let before = git(&box_.project(), &["rev-parse", "HEAD"]);
    let out = run(&box_, "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("staged or conflicted changes"),
        "{}",
        stderr(&out)
    );
    assert_eq!(git(&box_.project(), &["rev-parse", "HEAD"]), before);
    assert_eq!(
        git(&box_.project(), &["diff", "--cached", "--name-only"]).trim(),
        "a.txt"
    );
}

#[tokio::test]
async fn makes_two_reviewed_commits() {
    let box_ = setup("git-commit-two");
    let _server = provider(&box_).await;
    let out = tokio::task::spawn_blocking(move || {
        let result = run(&box_, "y\ncontext\nc\ny\ny\n");
        (result, box_)
    })
    .await
    .expect("flow thread");
    assert!(out.0.status.success(), "{}", stderr(&out.0));
    assert!(stdout(&out.0).contains("2 commit(s)"));
    assert_eq!(
        git(&out.1.project(), &["log", "--format=%s"])
            .lines()
            .take(2)
            .collect::<Vec<_>>(),
        vec!["feat: update b", "feat: update a"]
    );
    assert!(git(&out.1.project(), &["status", "--porcelain"]).is_empty());
}

#[tokio::test]
async fn declining_second_commit_keeps_first() {
    let box_ = setup("git-commit-stop");
    let _server = provider(&box_).await;
    let (out, box_) = tokio::task::spawn_blocking(move || {
        let result = run(&box_, "y\n\nc\ny\nn\n");
        (result, box_)
    })
    .await
    .expect("flow thread");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        git(&box_.project(), &["log", "-1", "--format=%s"]).trim(),
        "feat: update a"
    );
    assert!(git(&box_.project(), &["diff", "--cached", "--name-only"]).is_empty());
    assert!(git(&box_.project(), &["status", "--porcelain"]).contains("b.txt"));
}
