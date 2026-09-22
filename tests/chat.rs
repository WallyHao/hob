//! `hob.agent.open` conversations against a mock provider.

mod common;

use common::mock::{self, assistant};
use common::{Flow, stderr, stdout};
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_conversation_keeps_turns_and_usage() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("one")).await;
    let flow = mock::flow(
        "chat",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "test-model", system = "be brief" }
        hob.term.print((chat:send{ prompt = "first" }))
        hob.term.print((chat:send{ prompt = "second" }))
        hob.term.print(tostring(#chat:turns()))
        hob.term.print(tostring(chat:usage().total_tokens))
        chat:close()
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "one\none\n4\n10\n");

    let requests = server.received_requests().await.expect("requests");
    let second: Value = requests[1].body_json().expect("the body is JSON");
    assert_eq!(second["messages"][0]["role"], "system");
    assert_eq!(second["messages"].as_array().expect("messages").len(), 4);
}

#[tokio::test]
async fn reset_forgets_turns_but_keeps_usage() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("one")).await;
    let flow = mock::flow(
        "reset",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "m" }
        chat:send{ prompt = "first" }
        chat:reset()
        hob.term.print(tostring(#chat:turns()) .. " " .. tostring(chat:usage().total_tokens))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "0 5\n");
}

#[tokio::test]
async fn list_returns_model_ids() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{ "id": "a" }, { "id": "b" }]
        })))
        .mount(&server)
        .await;
    let flow = mock::flow(
        "list",
        &server,
        r#"hob.term.print(table.concat(hob.agent.list{ provider = provider }, ","))"#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "a,b\n");
}

#[test]
fn push_injects_messages_without_calling_the_model() {
    let flow = Flow::new(
        "push",
        r#"
        local chat = hob.agent.open{ model = "m" }
        chat:push("note")
        chat:push{ role = "tool", content = "42", tool_call_id = "call_1" }
        local turns = chat:turns()
        hob.term.print(turns[1].role .. " " .. turns[2].role .. " " .. turns[2].tool_call_id)
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "user tool call_1\n");
}

#[test]
fn a_closed_conversation_is_an_error() {
    let flow = Flow::new(
        "closed",
        r#"
        local chat = hob.agent.open{ model = "m" }
        chat:close()
        chat:turns()
        "#,
    );
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no conversation with handle 1"),
        "{}",
        stderr(&output)
    );
}
