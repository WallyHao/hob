//! The provider client against a local server: request shape, auth header and
//! error redaction. No test here touches the network.

use hob::provider::{ChatRequest, Client, Message, Protocol, ProviderSpec, Secret};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "sk-test-key-do-not-print";
const AUTH: &str = "Bearer sk-test-key-do-not-print";

fn client(server: &MockServer) -> Client {
    let spec = ProviderSpec::new("test", &server.uri(), "TEST_API_KEY", Protocol::OpenAi);
    Client::new(spec, Secret::new(KEY.to_owned())).expect("the client builds")
}

fn request() -> ChatRequest {
    ChatRequest::new("deepseek-chat", vec![Message::user("hi")])
}

#[tokio::test]
async fn chat_sends_the_key_as_a_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", AUTH))
        .and(body_json(json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": "hi"}],
            "stream": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"role": "assistant", "content": "hello"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server).chat(&request()).await.expect("200 answers");
    assert_eq!(response.first_text(), Some("hello"));
}

#[tokio::test]
async fn an_error_body_never_echoes_the_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string(format!("bad key: {KEY}")))
        .mount(&server)
        .await;

    let error = client(&server)
        .chat(&request())
        .await
        .expect_err("401 fails");
    let text = error.to_string();
    assert!(!text.contains(KEY), "{text}");
    assert!(text.contains("***"), "{text}");
}

#[tokio::test]
async fn models_returns_the_listed_ids() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("authorization", AUTH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "deepseek-chat"}, {"id": "deepseek-reasoner"}]
        })))
        .mount(&server)
        .await;

    let models = client(&server).models().await.expect("200 answers");
    assert_eq!(models, ["deepseek-chat", "deepseek-reasoner"]);
}
