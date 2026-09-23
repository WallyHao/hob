// --- exec::agent::wire ---
// The shapes an effect returns: the request it sends, the meta table, the
// usage table and the answer envelope.

use serde_json::{Map, Value, json};

use crate::effect::ops::agent::Settings;
use crate::provider::{ChatRequest, ChatResponse, Message, ProviderSpec, Usage};

use super::messages::flat_call;

/// One request from the conversation and the settings.
pub(crate) fn request(model: &str, messages: &[Message], settings: &Settings) -> ChatRequest {
    let mut request = ChatRequest::new(model, messages.to_vec());
    request.stream = settings.stream.unwrap_or(false);
    if request.stream {
        request.stream_options = Some(json!({ "include_usage": true }));
    }
    request.max_tokens = settings.max_tokens;
    request.temperature = settings.temperature;
    request.reasoning_effort.clone_from(&settings.effort);
    request.tools.clone_from(&settings.tools);
    request
}

/// The meta table an answer carries; `spent` is every attempt's usage, not
/// only the last one's, so a repaired answer reports what it really cost.
pub(crate) fn meta(
    spec: &ProviderSpec,
    model: &str,
    attempts: u32,
    response: &ChatResponse,
    spent: Usage,
) -> Value {
    let mut map = Map::new();
    map.insert("provider".to_owned(), json!(spec.id));
    map.insert("model".to_owned(), json!(model));
    map.insert("attempts".to_owned(), json!(attempts));
    map.insert("usage".to_owned(), usage(spent));
    if let Some(reason) = response
        .choices
        .first()
        .and_then(|choice| choice.finish_reason.as_deref())
    {
        map.insert("finish_reason".to_owned(), json!(reason));
    }
    if response.truncated() {
        map.insert("truncated".to_owned(), json!(true));
    }
    let message = response.choices.first().map(|choice| &choice.message);
    if let Some(calls) = message.and_then(|message| message.tool_calls.as_deref()) {
        let flat: Vec<Value> = calls.iter().map(flat_call).collect();
        map.insert("tool_calls".to_owned(), Value::Array(flat));
    }
    if let Some(reasoning) = message.and_then(|message| message.reasoning_content.as_deref()) {
        map.insert("reasoning".to_owned(), json!(reasoning));
    }
    Value::Object(map)
}

/// Token counts as a table.
pub(crate) fn usage(usage: Usage) -> Value {
    json!({
        "prompt_tokens": usage.prompt_tokens,
        "completion_tokens": usage.completion_tokens,
        "total_tokens": usage.total_tokens,
    })
}

/// The `{ answer, meta }` table an effect returns.
pub(crate) fn result(answer: &Value, meta: &Value) -> Value {
    json!({ "answer": answer, "meta": meta })
}
