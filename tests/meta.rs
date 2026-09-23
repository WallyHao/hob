//! What an answer says about itself: why it stopped, and what it cost.

mod common;

use common::mock::{self, assistant};
use common::{stderr, stdout};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Answer with `content`, stopped because the output cap was reached.
async fn truncated(server: &MockServer, content: &str) {
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{ "message": assistant(content), "finish_reason": "length" }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 2, "total_tokens": 3 }
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn a_truncated_answer_says_so() {
    let server = MockServer::start().await;
    truncated(&server, "half a sen").await;
    let flow = mock::flow(
        "truncated",
        &server,
        r#"
        local _, meta = hob.agent.ask{ provider = provider, model = "m", prompt = "x" }
        hob.term.print(meta.finish_reason .. " " .. tostring(meta.truncated))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "length true\n");
}

#[tokio::test]
async fn a_truncated_json_answer_names_the_cap() {
    let server = MockServer::start().await;
    truncated(&server, "half a js").await;
    let flow = mock::flow(
        "truncated-json",
        &server,
        r#"
        hob.agent.ask{
          provider = provider, model = "m", prompt = "x", max_attempts = 1,
          schema = { type = "object" },
        }
        "#,
    );
    let output = mock::run(flow).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stderr(&output).contains("cut off by the output cap"),
        "{}",
        stderr(&output)
    );
}
