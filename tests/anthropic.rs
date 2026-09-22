//! The Anthropic Messages dialect through a mock server.

mod common;

use common::{Flow, stderr, stdout};
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn flow(tag: &str, server: &MockServer, body: &str) -> Flow {
    let uri = server.uri();
    let source = format!(
        "local provider = {{ id = \"claude\", base_url = \"{uri}\", \
         api_key_env = \"TEST_KEY\", protocol = \"anthropic\" }}\n{body}"
    );
    Flow::new(tag, &source)
}

async fn reply(server: &MockServer, body: Value) {
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn a_request_speaks_the_messages_protocol() {
    let server = MockServer::start().await;
    reply(
        &server,
        json!({
            "content": [{ "type": "text", "text": "hello" }],
            "usage": { "input_tokens": 4, "output_tokens": 2 }
        }),
    )
    .await;
    let flow = flow(
        "anthropic-ask",
        &server,
        r#"
        local answer, meta = hob.agent.ask{ provider = provider, model = "claude-x", prompt = "hi", system = "be brief" }
        hob.term.print(answer .. " " .. tostring(meta.usage.total_tokens))
        "#,
    );
    let output = common::mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello 6\n");

    let requests = server.received_requests().await.expect("requests");
    let headers = &requests[0].headers;
    assert_eq!(headers["x-api-key"], "sk-test");
    assert_eq!(headers["anthropic-version"], "2023-06-01");
    assert!(headers.get("authorization").is_none());
    let body: Value = requests[0].body_json().expect("the body is JSON");
    assert_eq!(body["system"], "be brief");
    assert_eq!(body["max_tokens"], 4096);
    assert_eq!(body["messages"][0]["role"], "user");
}

#[tokio::test]
async fn tool_calls_and_tool_results_round_trip() {
    let server = MockServer::start().await;
    reply(
        &server,
        json!({
            "content": [
                { "type": "text", "text": "checking" },
                { "type": "tool_use", "id": "toolu_1", "name": "read", "input": { "path": "a" } }
            ],
            "usage": { "input_tokens": 5, "output_tokens": 3 }
        }),
    )
    .await;
    let flow = flow(
        "anthropic-tools",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "claude-x" }
        local answer, meta = chat:send{ prompt = "read a" }
        hob.term.print(answer .. " " .. meta.tool_calls[1].name .. " " .. meta.tool_calls[1].arguments)
        chat:push{ role = "tool", content = "contents", tool_call_id = "toolu_1" }; chat:send{ prompt = "summarize" }
        "#,
    );
    let output = common::mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "checking read {\"path\":\"a\"}\n");

    let requests = server.received_requests().await.expect("requests");
    let second: Value = requests[1].body_json().expect("the body is JSON");
    let result = &second["messages"][2]["content"][0];
    assert_eq!(second["messages"][1]["content"][1]["type"], "tool_use");
    assert_eq!(result["type"], "tool_result");
    assert_eq!(result["tool_use_id"], "toolu_1");
}

#[tokio::test]
async fn models_are_listed_through_the_messages_protocol() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("x-api-key", "sk-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{ "id": "claude-x" }, { "id": "claude-y" }]
        })))
        .mount(&server)
        .await;
    let flow = flow(
        "anthropic-models",
        &server,
        r#"hob.term.print(table.concat(hob.agent.list{ provider = provider }, ","))"#,
    );
    let output = common::mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "claude-x,claude-y\n");
}

#[test]
fn effort_is_refused_by_the_messages_protocol() {
    let flow = Flow::new(
        "anthropic-effort",
        r#"
        hob.agent.ask{
          provider = { id = "x", base_url = "http://127.0.0.1:9", api_key_env = "TEST_KEY", protocol = "anthropic" },
          model = "claude-x", prompt = "hi", effort = "high" }
        "#,
    );
    let output = flow.run_with(&[], &[("TEST_KEY", "sk-test")], None);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("does not support"), "{output:?}");
}
