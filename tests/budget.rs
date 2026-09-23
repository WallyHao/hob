//! The run budgets: `--max-calls` and `--max-tokens` stop a flow that spends.

mod common;

use common::mock::{self, assistant};
use common::stderr;
use wiremock::MockServer;

const ONE_ASK: &str = r#"hob.agent.ask{ provider = provider, model = "m", prompt = "one" }"#;

#[tokio::test]
async fn the_call_cap_refuses_the_next_call() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow("max-calls", &server, &format!("{ONE_ASK}\n{ONE_ASK}\n"));
    let output = mock::run_args(flow, &["--max-calls", "1"]).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stderr(&output).contains("call budget"),
        "{}",
        stderr(&output)
    );
    assert_eq!(server.received_requests().await.expect("requests").len(), 1);
}

#[tokio::test]
async fn the_token_cap_stops_the_next_call() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow("max-tokens", &server, &format!("{ONE_ASK}\n{ONE_ASK}\n"));
    let output = mock::run_args(flow, &["--max-tokens", "4"]).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stderr(&output).contains("token budget"),
        "{}",
        stderr(&output)
    );
    assert_eq!(server.received_requests().await.expect("requests").len(), 1);
}

#[tokio::test]
async fn a_zero_cap_is_no_cap() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow("max-zero", &server, &format!("{ONE_ASK}\n{ONE_ASK}\n"));
    let output = mock::run_args(flow, &["--max-calls=0", "--max-tokens=0"]).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(server.received_requests().await.expect("requests").len(), 2);
}
