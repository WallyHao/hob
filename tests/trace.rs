//! `--trace`: the JSONL record of every effect a run requested and settled.

mod common;

use common::{Flow, stderr};

fn records(flow: &Flow) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(flow.dir().join("trace.jsonl")).expect("trace file");
    text.lines()
        .map(|line| serde_json::from_str(line).expect("one JSON object per line"))
        .collect()
}

#[test]
fn a_trace_records_requests_and_outcomes() {
    let flow = Flow::new("trace-ok", r#"hob.file.write("out.txt", "hi")"#);
    let output = flow.run(&["--trace", "trace.jsonl"]);
    assert!(output.status.success(), "{output:?}");
    let records = records(&flow);
    let request = records
        .iter()
        .find(|record| record["event"] == "request")
        .expect("a request record");
    assert_eq!(request["ns"], "file");
    assert_eq!(request["op"], "write");
    assert_eq!(request["cmd"]["path"], "out.txt");
    let outcome = records
        .iter()
        .find(|record| record["event"] == "outcome")
        .expect("an outcome record");
    assert_eq!(outcome["outcome"], "ok");
    assert!(records.iter().all(|record| record["t"].is_number()));
    assert_eq!(request["id"], outcome["id"]);
}

#[test]
fn a_refused_effect_is_recorded_as_dry_run() {
    let flow = Flow::new("trace-dry", r#"hob.file.write("out.txt", "hi")"#);
    let output = flow.run(&["--dry-run", "--trace", "trace.jsonl"]);
    assert!(output.status.success(), "{output:?}");
    assert!(!flow.dir().join("out.txt").exists());
    let records = records(&flow);
    let outcome = records
        .iter()
        .find(|record| record["event"] == "outcome")
        .expect("an outcome record");
    assert_eq!(outcome["outcome"], "dry-run");
}

#[test]
fn a_trace_masks_a_secret_from_the_environment() {
    let flow = Flow::new(
        "trace-secret",
        r#"
        local _, err = hob.effect(
          "file", "read", { path = hob.args[1] }, { fallible = true }
        )
        hob.term.print(err)
        "#,
    );
    let output = flow.run_with(
        &["--trace", "trace.jsonl", "--trace-full", "supersecret123"],
        &[("TEST_API_KEY", "supersecret123")],
        None,
    );
    assert!(output.status.success(), "{output:?}");
    let text = std::fs::read_to_string(flow.dir().join("trace.jsonl")).expect("trace file");
    assert!(text.contains("***"), "{text}");
    assert!(!text.contains("supersecret123"), "{text}");
}

#[test]
fn a_trace_file_is_overwritten() {
    let flow = Flow::new("trace-over", r#"hob.term.print("x")"#);
    flow.write("trace.jsonl", "old\n");
    let output = flow.run(&["--trace", "trace.jsonl"]);
    assert!(output.status.success(), "{output:?}");
    let text = std::fs::read_to_string(flow.dir().join("trace.jsonl")).expect("trace file");
    assert!(!text.contains("old"), "{text}");
}

#[test]
fn an_unwritable_trace_fails_the_run() {
    let flow = Flow::new("trace-fail", r#"hob.term.print("x")"#);
    let output = flow.run(&["--trace", "missing/trace.jsonl"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("cannot write"), "{output:?}");
}

#[test]
fn content_is_recorded_as_a_size_by_default() {
    let flow = Flow::new(
        "trace-content",
        r#"hob.file.write("out.txt", "secret body")"#,
    );
    let output = flow.run(&["--trace", "trace.jsonl"]);
    assert!(output.status.success(), "{output:?}");
    let text = std::fs::read_to_string(flow.dir().join("trace.jsonl")).expect("trace file");
    assert!(!text.contains("secret body"), "{text}");
    assert!(text.contains("<11 bytes>"), "{text}");
}

#[test]
fn trace_full_records_the_content() {
    let flow = Flow::new("trace-full", r#"hob.file.write("out.txt", "secret body")"#);
    let output = flow.run(&["--trace", "trace.jsonl", "--trace-full"]);
    assert!(output.status.success(), "{output:?}");
    let text = std::fs::read_to_string(flow.dir().join("trace.jsonl")).expect("trace file");
    assert!(text.contains("secret body"), "{text}");
}

#[test]
fn trace_full_needs_a_trace() {
    let flow = Flow::new("trace-full-alone", r#"hob.term.print("x")"#);
    let output = flow.run(&["--trace-full"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--trace"), "{output:?}");
}

#[test]
fn trace_needs_a_file() {
    let flow = Flow::new("trace-arg", r#"hob.term.print("x")"#);
    let output = flow.run(&["--trace"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("needs a file"), "{output:?}");
}
