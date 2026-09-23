// --- exec::skipped ---
// What a refused effect resumes the flow with.
//
// The values are the conservative answer, shaped like a real success so the
// rest of the flow can run and a preview shows its whole path: a refused
// command reports success with no output, a refused question takes its default,
// and a refused model call answers with an example built from its schema. A
// flow that branches on one of these may take a surprising path, which is why
// every refusal is announced. `--yes` answers questions with the same values.

use serde_json::{Value, json};

use crate::effect::Request;
use crate::effect::ops::agent::{Ask, Send};
use crate::effect::ops::term::{Choose, Select};
use crate::exec::agent::{Chat, schema, wire};
use crate::exec::{State, decode, term};
use crate::provider::Usage;

/// The value a refused request stands in for.
pub(crate) fn result(state: &State, request: &Request) -> Value {
    match (request.ns.as_str(), request.op.as_str()) {
        ("agent", "ask") => ask(request),
        ("agent", "send") => send(state, request),
        ("agent", "list") => Value::Array(Vec::new()),
        ("proc", "exec" | "shell") => quiet_success(),
        ("term", "allow") => Value::Bool(bool_field(request, "default")),
        ("term", "input") => Value::String(text_field(request, "default")),
        ("term", "select") => decode::<Select>(request).map_or_else(
            |_| Value::String(String::new()),
            |select| term::select_default(&select),
        ),
        ("term", "choose") => decode::<Choose>(request).map_or_else(
            |_| Value::Array(Vec::new()),
            |choose| term::choose_default(&choose),
        ),
        _ => Value::Null,
    }
}

fn ask(request: &Request) -> Value {
    let answer = decode::<Ask>(request)
        .ok()
        .and_then(|ask| ask.settings.schema)
        .map_or_else(
            || Value::String(String::new()),
            |schema| schema::example(&schema),
        );
    wire::result(&answer, &meta(provider(request), model(request)))
}

fn send(state: &State, request: &Request) -> Value {
    let session = request.cmd.get("session").and_then(Value::as_u64);
    let (provider, model) = session
        .and_then(|id| state.chats.get(&id))
        .map_or(("", ""), Chat::label);
    let answer = decode::<Send>(request)
        .ok()
        .and_then(|send| send.settings.schema)
        .map_or_else(
            || Value::String(String::new()),
            |schema| schema::example(&schema),
        );
    wire::result(&answer, &meta(provider, model))
}

/// A refused command reports success with no output.
fn quiet_success() -> Value {
    json!({
        "code": 0,
        "stdout": "",
        "stderr": "",
        "ok": true,
        "duration_ms": 0,
        "truncated": false,
    })
}

fn meta(provider: &str, model: &str) -> Value {
    json!({
        "provider": provider,
        "model": model,
        "attempts": 0,
        "usage": wire::usage(Usage::default()),
    })
}

fn provider(request: &Request) -> &str {
    match request.cmd.get("provider") {
        Some(Value::String(id)) => id,
        Some(Value::Object(spec)) => spec.get("id").and_then(Value::as_str).unwrap_or("inline"),
        _ => "",
    }
}

fn model(request: &Request) -> &str {
    request
        .cmd
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn text_field(request: &Request, key: &str) -> String {
    request
        .cmd
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn bool_field(request: &Request, key: &str) -> bool {
    request
        .cmd
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
