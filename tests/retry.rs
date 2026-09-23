//! Retry policy and the answer cap: only a failure that could clear is
//! attempted again, and a runaway body cannot grow the heap.

mod common;

use common::{Flow, mock, stderr};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Answer every chat completion with `status`, and expect it `calls` times.
async fn failing(server: &MockServer, status: u16, calls: u64) {
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(status).set_body_json(json!({ "error": "no" })))
        .expect(calls)
        .mount(server)
        .await;
}

fn flow(tag: &str, server: &MockServer) -> Flow {
    mock::flow(
        tag,
        server,
        r#"hob.agent.ask{ provider = provider, model = "m", prompt = "hi" }"#,
    )
}

#[tokio::test]
async fn a_rejected_request_is_not_retried() {
    let server = MockServer::start().await;
    failing(&server, 401, 1).await;

    // The mock's `expect(1)` fails the test if the client asks again.
    let output = mock::run(flow("no-retry", &server)).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("401"), "{output:?}");
}

#[tokio::test]
async fn a_server_error_is_retried() {
    let server = MockServer::start().await;
    failing(&server, 503, 3).await;

    let output = mock::run(flow("retry", &server)).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("503"), "{output:?}");
}

#[tokio::test]
async fn an_oversized_answer_is_refused() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(11 * 1024 * 1024)))
        .mount(&server)
        .await;

    let output = mock::run(flow("too-big", &server)).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("more than"), "{output:?}");
}
