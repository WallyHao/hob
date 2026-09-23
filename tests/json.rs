//! `--json`: the machine-readable answers of the describing verbs.

mod common;

use common::{Sandbox, stdout};

/// The one object a `--json` verb prints.
fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output)).expect("one JSON object")
}

#[test]
fn list_json_describes_the_commands() {
    let sandbox = Sandbox::new("json-list");
    sandbox.write_project("hello", "--- Say hello.\n");
    let output = sandbox.run_project(&["list", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    let hello = value["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|command| command["name"] == "hello")
        .expect("hello");
    assert_eq!(hello["origin"], "project");
    assert_eq!(hello["summary"], "Say hello.");
    assert_eq!(hello["trusted"], true);
    assert!(hello["path"].is_string(), "{hello}");
    assert!(value["ignored"].is_array(), "{value}");
}

#[test]
fn list_json_marks_an_untrusted_command() {
    let sandbox = Sandbox::untrusted("json-untrusted");
    sandbox.write_project("hello", "");
    let output = sandbox.run_project(&["list", "--json"]);
    let hello = json(&output)["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|command| command["name"] == "hello")
        .expect("hello")
        .clone();
    assert_eq!(hello["trusted"], false);
}

#[test]
fn which_json_lists_the_layers() {
    let sandbox = Sandbox::new("json-which");
    sandbox.write_project("deploy", "");
    sandbox.write_user("deploy", "");
    let output = sandbox.run_project(&["which", "deploy", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["name"], "deploy");
    let layers = value["layers"].as_array().expect("layers");
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0]["origin"], "project");
    assert_eq!(layers[0]["effective"], true);
    assert_eq!(layers[1]["origin"], "user");
    assert_eq!(layers[1]["effective"], false);
}

#[test]
fn trust_list_json_names_the_projects() {
    let sandbox = Sandbox::new("json-trust");
    let output = sandbox.run_project(&["trust", "--list", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let trusted = json(&output)["trusted"]
        .as_array()
        .expect("trusted")
        .clone();
    let root = sandbox
        .project()
        .canonicalize()
        .expect("the project resolves");
    assert_eq!(trusted, vec![serde_json::json!(root)], "{trusted:?}");
}

#[test]
fn doctor_json_reports_paths_and_keys() {
    let sandbox = Sandbox::new("json-doctor");
    let output = sandbox.run_project(&["doctor", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert!(
        value["version"]
            .as_str()
            .expect("version")
            .starts_with("hob "),
        "{value}"
    );
    assert_eq!(value["commands"]["builtin"], 1);
    assert_eq!(value["trust"]["trusted"], true);
    assert!(
        value["keys"]
            .as_array()
            .expect("keys")
            .iter()
            .any(|key| key["id"] == "deepseek"),
        "{value}"
    );
}
