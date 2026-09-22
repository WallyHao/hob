//! Tool calls and reasoning through `hob.agent.ask`.

mod common;

use common::mock;
use common::stdout;
use serde_json::{Value, json};
use wiremock::MockServer;

#[tokio::test]
async fn tools_are_data() {
    let server = MockServer::start().await;
    mock::reply(
        &server,
        json!({
            "role": "assistant",
            "content": "",
            "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": { "name": "lookup", "arguments": "{\"q\":\"x\"}" }
            }]
        }),
    )
    .await;
    let flow = mock::flow(
        "tools",
        &server,
        r#"
        local answer, meta = hob.agent.ask{
          provider = provider, model = "m", prompt = "find x",
          tools = { { type = "function", ["function"] = { name = "lookup", parameters = { type = "object" } } } },
        }
        hob.term.print(meta.tool_calls[1].name .. " " .. meta.tool_calls[1].arguments)
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "lookup {\"q\":\"x\"}\n");

    let requests = server.received_requests().await.expect("requests");
    let body: Value = requests[0].body_json().expect("the body is JSON");
    assert_eq!(body["tools"][0]["function"]["name"], "lookup");
}

#[tokio::test]
async fn reasoning_is_reported() {
    let server = MockServer::start().await;
    mock::reply(
        &server,
        json!({ "role": "assistant", "content": "42", "reasoning_content": "because" }),
    )
    .await;
    let flow = mock::flow(
        "reasoning",
        &server,
        r#"
        local answer, meta = hob.agent.ask{ provider = provider, model = "m", prompt = "why?" }
        hob.term.print(answer .. " " .. meta.reasoning)
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "42 because\n");
}
