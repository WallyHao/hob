//! `hob.agent.ask` against a mock provider: answers, schemas and the errors
//! that never reach the network.

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};

use common::mock::{self, assistant};
use common::{Flow, stderr, stdout};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

#[tokio::test]
async fn ask_returns_text_and_meta() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("hello")).await;
    let flow = mock::flow(
        "ask",
        &server,
        r#"
        local answer, meta = hob.agent.ask{ provider = provider, model = "test-model", prompt = "hi" }
        hob.term.print(answer)
        hob.term.print(meta.provider .. " " .. meta.model .. " " .. tostring(meta.attempts) .. " " .. tostring(meta.usage.total_tokens))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello\ntest test-model 1 5\n");
}

/// Answers the first call with prose and every later one with JSON.
struct Repair(AtomicUsize);

impl Respond for Repair {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let content = if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            "not json"
        } else {
            "{\"name\":\"wally\"}"
        };
        ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{ "message": { "role": "assistant", "content": content } }]
        }))
    }
}

#[tokio::test]
async fn a_schema_is_validated_and_repaired() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(Repair(AtomicUsize::new(0)))
        .mount(&server)
        .await;
    let flow = mock::flow(
        "repair",
        &server,
        r#"
        local answer, meta = hob.agent.ask{
          provider = provider, model = "test-model", prompt = "who?",
          schema = { type = "object", required = { "name" }, properties = { name = { type = "string" } } },
        }
        hob.term.print(answer.name)
        hob.term.print(tostring(meta.attempts))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "wally\n2\n");
}

#[tokio::test]
async fn a_schema_that_never_matches_fails() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("still not json")).await;
    let flow = mock::flow(
        "never",
        &server,
        r#"
        hob.agent.ask{
          provider = provider, model = "m", prompt = "x", max_attempts = 2,
          schema = { type = "object" },
        }
        "#,
    );
    let output = mock::run(flow).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("valid JSON"),
        "{}",
        stderr(&output)
    );
}

#[tokio::test]
async fn a_missing_key_names_its_variable() {
    let server = MockServer::start().await;
    let flow = mock::flow(
        "nokey",
        &server,
        r#"hob.agent.ask{ provider = provider, model = "m", prompt = "hi" }"#,
    );
    let output = mock::run_with(flow, Vec::new()).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("TEST_KEY"), "{}", stderr(&output));
}

#[test]
fn an_unknown_provider_is_an_error() {
    let flow = Flow::new(
        "unknown",
        r#"hob.agent.ask{ provider = "nope", model = "m", prompt = "hi" }"#,
    );
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("unknown provider `nope`"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_missing_model_says_so() {
    let flow = Flow::new("nomodel", r#"hob.agent.ask{ prompt = "hi" }"#);
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("no model"), "{}", stderr(&output));
}
