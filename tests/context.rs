//! The prompt budget: a long conversation drops its oldest exchanges.

mod common;

use common::mock::{self, assistant};
use common::stdout;
use serde_json::Value;
use wiremock::MockServer;

#[tokio::test]
async fn an_old_exchange_is_dropped_before_a_send() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow(
        "budget",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "m", max_prompt_tokens = 40 }
        chat:send{ prompt = "first question padded out to be long" }
        chat:send{ prompt = "second question padded out to be long" }
        local _, meta = chat:send{ prompt = "third question padded out to be long" }
        hob.term.print(tostring(meta.trimmed))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "2\n");

    let requests = server.received_requests().await.expect("requests");
    let last: Value = requests[2].body_json().expect("the body is JSON");
    let messages = last["messages"].as_array().expect("messages");
    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(roles, ["system", "user", "assistant", "user"], "{last}");
    assert!(
        messages[0]["content"]
            .as_str()
            .is_some_and(|marker| marker.contains("dropped 2")),
        "{last}"
    );
    assert!(
        messages[1]["content"]
            .as_str()
            .is_some_and(|first| first.starts_with("second")),
        "{last}"
    );
}

#[tokio::test]
async fn a_tool_result_never_leaves_its_call() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow(
        "budget-tools",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "m", max_prompt_tokens = 60 }
        chat:push{ role = "user", content = "one" }
        chat:push{ role = "assistant", content = "call", tool_calls = { { id = "c1", name = "f", arguments = "{}" } } }
        chat:push{ role = "tool", content = "r1", tool_call_id = "c1" }
        chat:push{ role = "user", content = "two" }
        chat:push{ role = "assistant", content = "call", tool_calls = { { id = "c2", name = "f", arguments = "{}" } } }
        chat:push{ role = "tool", content = "r2", tool_call_id = "c2" }
        local _, meta = chat:send{ prompt = "three" }
        hob.term.print(tostring(meta.trimmed))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "3\n");

    let requests = server.received_requests().await.expect("requests");
    let last: Value = requests[0].body_json().expect("the body is JSON");
    let messages = last["messages"].as_array().expect("messages");
    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        roles,
        ["system", "user", "assistant", "tool", "user"],
        "{last}"
    );
    assert_eq!(messages[3]["tool_call_id"], "c2", "{last}");
}

#[tokio::test]
async fn zero_turns_the_cap_off() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("ok")).await;
    let flow = mock::flow(
        "budget-off",
        &server,
        r#"
        local chat = hob.agent.open{ provider = provider, model = "m", max_prompt_tokens = 0 }
        chat:send{ prompt = "first question padded out to be long" }
        local _, meta = chat:send{ prompt = "second question padded out to be long" }
        hob.term.print(tostring(meta.trimmed))
        "#,
    );
    let output = mock::run(flow).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "nil\n");

    let requests = server.received_requests().await.expect("requests");
    let last: Value = requests[1].body_json().expect("the body is JSON");
    assert_eq!(
        last["messages"].as_array().expect("messages").len(),
        3,
        "{last}"
    );
}
