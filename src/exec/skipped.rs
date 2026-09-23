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

use crate::effect::{Operation, Request};
use crate::exec::agent::{Chat, schema, wire};
use crate::exec::{RunContext, term};
use crate::provider::Usage;

/// The value a refused request stands in for.
pub(crate) fn result(state: &RunContext, request: &Request, operation: &Operation) -> Value {
    match operation {
        Operation::AgentAsk(args) => answer(
            args.settings.schema.as_ref(),
            provider(request),
            model(request),
        ),
        Operation::AgentSend(args) => {
            let chat = state.chats.get(&args.session);
            let (provider, model) = chat.map_or(("", ""), Chat::label);
            let schema = args
                .settings
                .schema
                .as_ref()
                .or_else(|| chat.and_then(Chat::schema));
            answer(schema, provider, model)
        }
        Operation::AgentList(..) => Value::Array(Vec::new()),
        Operation::ProcExec(..) | Operation::ProcShell(..) => quiet_success(),
        Operation::TermAllow(args) => Value::Bool(args.default.unwrap_or(false)),
        Operation::TermInput(args) => Value::String(
            args.default
                .clone()
                .or_else(|| args.initial.clone())
                .unwrap_or_default(),
        ),
        Operation::TermSelect(args) => term::select_default(args),
        Operation::TermChoose(args) => term::choose_default(args),
        Operation::FileRead(..)
        | Operation::FileWrite(..)
        | Operation::FileStat(..)
        | Operation::FileList(..)
        | Operation::ProcOpen(..)
        | Operation::ProcWhich(..)
        | Operation::ProcSetenv(..)
        | Operation::ProcUnset(..)
        | Operation::ProcChdir(..)
        | Operation::ProcSetup(..)
        | Operation::ProcState(..)
        | Operation::ProcReset(..)
        | Operation::ProcClose(..)
        | Operation::AgentOpen(..)
        | Operation::AgentPush(..)
        | Operation::AgentTurns(..)
        | Operation::AgentUsage(..)
        | Operation::AgentReset(..)
        | Operation::AgentClose(..)
        | Operation::TermPrint(..)
        | Operation::LogsWrite(..)
        | Operation::TmplFetch(..) => Value::Null,
    }
}

fn answer(schema: Option<&Value>, provider: &str, model: &str) -> Value {
    let answer = schema.map_or_else(|| Value::String(String::new()), schema::example);
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
