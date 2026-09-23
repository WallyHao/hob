//! A wiremock-backed provider for the agent tests.
//!
//! The flow runs in a subprocess, so it must not run on the test's runtime
//! thread: `run` moves it to the blocking pool, leaving the mock server free to
//! answer.

use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::Flow;

/// The key the inline provider in `flow` reads.
const KEY: &str = "sk-test";

/// A flow whose inline provider points at the mock server, so `provider` is in
/// scope for the body.
pub(crate) fn flow(tag: &str, server: &MockServer, body: &str) -> Flow {
    let uri = server.uri();
    let source = format!(
        "local provider = {{ id = \"test\", base_url = \"{uri}\", api_key_env = \"TEST_KEY\" }}\n{body}"
    );
    Flow::new(tag, &source)
}

/// Answer every chat completion with one message body and fixed usage.
pub(crate) async fn reply(server: &MockServer, message: Value) {
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{ "message": message }],
            "usage": { "prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5 }
        })))
        .mount(server)
        .await;
}

/// An assistant message body.
pub(crate) fn assistant(content: &str) -> Value {
    json!({ "role": "assistant", "content": content })
}

/// Run the flow with the provider key present.
pub(crate) async fn run(flow: Flow) -> std::process::Output {
    run_with(flow, vec![("TEST_KEY".to_owned(), KEY.to_owned())]).await
}

/// Run the flow with control flags and the provider key present.
pub(crate) async fn run_args(flow: Flow, args: &'static [&'static str]) -> std::process::Output {
    tokio::task::spawn_blocking(move || flow.run_with(args, &[("TEST_KEY", KEY)], None))
        .await
        .expect("the flow thread finishes")
}

/// Run the flow with an explicit environment.
pub(crate) async fn run_with(flow: Flow, env: Vec<(String, String)>) -> std::process::Output {
    tokio::task::spawn_blocking(move || {
        let pairs: Vec<(&str, &str)> = env
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        flow.run_with(&[], &pairs, None)
    })
    .await
    .expect("the flow thread finishes")
}
