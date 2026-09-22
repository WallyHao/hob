// --- exec::agent::messages ---
// Translating between the flow's tables and the provider's message types.
// Tool calls are flat in a flow (`{ id, name, arguments }`) and nested on the
// wire, because the flow wants the name first and the wire wants the type.

use serde_json::{Map, Value, json};

use crate::effect::Failure;
use crate::provider::{FunctionCall, Message, Role, ToolCall};

/// Build the opening conversation from `system`, `prompt` or `messages`.
pub(crate) fn opening(
    messages: Option<&Vec<Value>>,
    prompt: Option<&str>,
    system: Option<&str>,
) -> Result<Vec<Message>, Failure> {
    let mut out = Vec::new();
    if let Some(system) = system {
        out.push(Message::system(system));
    }
    match (prompt, messages) {
        (Some(_), Some(_)) => Err(Failure::new("pass either `prompt` or `messages`, not both")),
        (Some(prompt), None) => {
            out.push(Message::user(prompt));
            Ok(out)
        }
        (None, Some(messages)) => {
            if messages.is_empty() {
                return Err(Failure::new("`messages` is empty"));
            }
            for message in messages {
                out.push(message_of(message)?);
            }
            Ok(out)
        }
        (None, None) => Err(Failure::new("pass `prompt` or `messages`")),
    }
}

/// One message from a flow table: `{ role, content, tool_calls, tool_call_id }`.
pub(crate) fn message_of(value: &Value) -> Result<Message, Failure> {
    let fields = value
        .as_object()
        .ok_or_else(|| Failure::new("a message must be a table"))?;
    let role = match fields.get("role").and_then(Value::as_str).unwrap_or("user") {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        other => return Err(Failure::new(format!("unknown role `{other}`"))),
    };
    Ok(Message {
        role,
        content: fields
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        tool_calls: tool_calls_of(fields.get("tool_calls"))?,
        tool_call_id: fields
            .get("tool_call_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        reasoning_content: None,
    })
}

/// One committed message as the flow sees it.
pub(crate) fn turn(message: &Message) -> Value {
    let mut map = Map::new();
    map.insert("role".to_owned(), json!(message.role.name()));
    map.insert("content".to_owned(), json!(message.content));
    if let Some(calls) = &message.tool_calls {
        let flat: Vec<Value> = calls.iter().map(flat_call).collect();
        map.insert("tool_calls".to_owned(), Value::Array(flat));
    }
    if let Some(id) = &message.tool_call_id {
        map.insert("tool_call_id".to_owned(), json!(id));
    }
    Value::Object(map)
}

/// A wire tool call as the flow sees it.
pub(crate) fn flat_call(call: &ToolCall) -> Value {
    json!({
        "id": call.id,
        "name": call.function.name,
        "arguments": call.function.arguments,
    })
}

/// Flat `{ id, name, arguments }` entries from a flow become wire tool calls.
fn tool_calls_of(value: Option<&Value>) -> Result<Option<Vec<ToolCall>>, Failure> {
    let Some(value) = value else {
        return Ok(None);
    };
    let calls = value
        .as_array()
        .ok_or_else(|| Failure::new("`tool_calls` must be a list"))?;
    let mut out = Vec::new();
    for call in calls {
        let fields = call
            .as_object()
            .ok_or_else(|| Failure::new("a tool call must be a table"))?;
        let name = fields
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| Failure::new("a tool call needs a `name`"))?;
        let arguments = match fields.get("arguments") {
            Some(Value::String(text)) => text.clone(),
            Some(other) => other.to_string(),
            None => "{}".to_owned(),
        };
        out.push(ToolCall {
            id: fields
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            kind: "function".to_owned(),
            function: FunctionCall {
                name: name.to_owned(),
                arguments,
            },
        });
    }
    Ok(Some(out))
}
