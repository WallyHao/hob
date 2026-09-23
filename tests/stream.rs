//! Streaming answers: deltas on stderr, the assembled answer returned whole.

mod common;

use common::mock;
use common::{Flow, stderr, stdout};
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const OPENAI: &str = concat!(
    "data: {\"choices\":[{\"delta\":{\"content\":\"he\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"delta\":{\"content\":\"llo\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
    "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5}}\n\n",
    "data: [DONE]\n\n",
);

const ANTHROPIC: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":4,\"output_tokens\":0}}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"bonjour\"}}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

async fn events(server: &MockServer, endpoint: &str, body: &'static str) {
    Mock::given(method("POST"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .mount(server)
        .await;
}

#[tokio::test]
async fn openai_deltas_arrive_and_the_answer_is_assembled() {
    let server = MockServer::start().await;
    events(&server, "/chat/completions", OPENAI).await;
    let flow = mock::flow(
        "stream",
        &server,
        r#"
        local answer, meta = hob.agent.ask{ provider = provider, model = "m", prompt = "hi", stream = true }
        hob.term.print(answer)
        hob.term.print(meta.finish_reason .. " " .. tostring(meta.usage.total_tokens))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello\nstop 5\n");
    assert!(stderr(&output).contains("hello"), "{}", stderr(&output));

    let requests = server.received_requests().await.expect("requests");
    let body: Value = requests[0].body_json().expect("the body is JSON");
    assert_eq!(body["stream"], true, "{body}");
    assert_eq!(body["stream_options"]["include_usage"], true, "{body}");
}

#[tokio::test]
async fn anthropic_deltas_arrive_and_the_answer_is_assembled() {
    let server = MockServer::start().await;
    events(&server, "/v1/messages", ANTHROPIC).await;
    let uri = server.uri();
    let source = format!(
        "local provider = {{ id = \"test\", base_url = \"{uri}\", api_key_env = \"TEST_KEY\", protocol = \"anthropic\" }}\n\
         local answer, meta = hob.agent.ask{{ provider = provider, model = \"m\", prompt = \"hi\", stream = true }}\n\
         hob.term.print(answer .. \" \" .. meta.finish_reason .. \" \" .. tostring(meta.usage.total_tokens))\n"
    );
    let output = mock::run(Flow::new("stream-anthropic", &source)).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "bonjour end_turn 6\n");

    let requests = server.received_requests().await.expect("requests");
    let body: Value = requests[0].body_json().expect("the body is JSON");
    assert_eq!(body["stream"], true, "{body}");
}

#[tokio::test]
async fn streaming_tools_are_refused() {
    let server = MockServer::start().await;
    let flow = mock::flow(
        "stream-tools",
        &server,
        r#"
        hob.agent.ask{
          provider = provider, model = "m", prompt = "hi", stream = true,
          tools = { { type = "function", ["function"] = { name = "f" } } },
        }
        "#,
    );
    let output = mock::run(flow).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        stderr(&output).contains("cannot be combined"),
        "{}",
        stderr(&output)
    );
}
