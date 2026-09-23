//! Providers defined in `config.toml`.

mod common;

use std::process::Command;

use common::mock::{self, assistant};
use common::{Flow, stderr, stdout};
use wiremock::MockServer;

#[tokio::test]
async fn a_provider_from_the_config_file_is_reachable() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("from config")).await;
    let flow = Flow::new(
        "config-provider",
        r#"hob.term.print((hob.agent.ask{ provider = "local", model = "m", prompt = "hi" }))"#,
    );
    flow.write(
        "config/config.toml",
        &format!(
            "[providers.local]\nbase_url = \"{}\"\napi_key_env = \"TEST_KEY\"\n",
            server.uri()
        ),
    );
    let output = mock::run_with(flow, vec![("TEST_KEY".to_owned(), "sk-test".to_owned())]).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "from config\n");
}

#[tokio::test]
async fn the_defaults_section_supplies_provider_and_model() {
    let server = MockServer::start().await;
    mock::reply(&server, assistant("defaulted")).await;
    let flow = Flow::new(
        "config-defaults",
        r#"hob.term.print((hob.agent.ask{ prompt = "hi" }))"#,
    );
    flow.write(
        "config/config.toml",
        &format!(
            "[defaults]\nprovider = \"local\"\nmodel = \"default-model\"\n\
             [providers.local]\nbase_url = \"{}\"\napi_key_env = \"TEST_KEY\"\n",
            server.uri()
        ),
    );
    let output = mock::run_with(flow, vec![("TEST_KEY".to_owned(), "sk-test".to_owned())]).await;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "defaulted\n");

    let requests = server.received_requests().await.expect("requests");
    assert_eq!(
        requests[0].body_json::<serde_json::Value>().unwrap()["model"],
        "default-model"
    );
}

#[test]
fn a_broken_config_file_is_reported() {
    let flow = Flow::new(
        "config-broken",
        r#"hob.agent.ask{ provider = "local", model = "m", prompt = "hi" }"#,
    );
    flow.write("config/config.toml", "providers = [oops");
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("config.toml"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_registry_wins_over_the_config_file() {
    let flow = Flow::new(
        "config-shadow",
        r#"hob.agent.ask{ provider = "deepseek", model = "m", prompt = "hi" }"#,
    );
    flow.write(
        "config/config.toml",
        "[providers.deepseek]\nbase_url = \"http://127.0.0.1:9\"\napi_key_env = \"CONFIG_KEY\"\n",
    );
    let output = Command::new(common::BIN)
        .env("HOB_CONFIG_DIR", flow.config())
        .env_remove("DEEPSEEK_API_KEY")
        .current_dir(flow.dir())
        .arg("run")
        .arg("flow.lua")
        .output()
        .expect("spawn hob");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("DEEPSEEK_API_KEY"),
        "{}",
        stderr(&output)
    );
}
