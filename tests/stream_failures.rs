//! Streaming failures must stay explicit and must not duplicate visible output.

mod common;

use common::{mock, stderr};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FLOW: &str =
    r#"hob.agent.ask{ provider = provider, model = "m", prompt = "hi", stream = true }"#;

#[tokio::test]
async fn a_failure_after_output_is_not_retried() {
    let server = MockServer::start().await;
    let broken = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"he\"},\"finish_reason\":null}]}\n\n",
        "data: not-json\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(broken, "text/event-stream"))
        .expect(1)
        .mount(&server)
        .await;
    let output = mock::run(mock::flow("stream-partial-failure", &server, FLOW)).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(stderr(&output).matches("he").count(), 1, "{output:?}");
}

#[tokio::test]
async fn an_incomplete_stream_is_an_error() {
    let server = MockServer::start().await;
    let unfinished =
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n";
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(unfinished, "text/event-stream"))
        .mount(&server)
        .await;
    let output = mock::run(mock::flow("stream-incomplete", &server, FLOW)).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stderr(&output).contains("completion marker"), "{output:?}");
}
