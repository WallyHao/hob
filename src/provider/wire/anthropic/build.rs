// --- provider::wire::anthropic::build ---
// Turning the engine's messages into a Messages request: a system prompt split
// out of the conversation, typed content blocks, and tools whose arguments are
// objects rather than JSON strings.

use serde_json::{Value, json};

use crate::provider::error::Error;
use crate::provider::types::{ChatRequest, Message, Role};

/// What a request without `max_tokens` uses; the field is required here.
const MAX_TOKENS: u32 = 4096;

/// Build a Messages request.
pub(crate) fn request(provider: &str, request: &ChatRequest) -> Result<Value, Error> {
    if request.reasoning_effort.is_some() {
        return Err(Error::Unsupported {
            provider: provider.to_owned(),
            option: "effort".to_owned(),
        });
    }
    let (system, messages) = split(&request.messages);
    let mut body = json!({
        "model": request.model,
        "max_tokens": request.max_tokens.unwrap_or(MAX_TOKENS),
        "messages": messages,
    });
    if request.stream {
        body["stream"] = json!(true);
    }
    if let Some(system) = system {
        body["system"] = json!(system);
    }
    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }
    if let Some(tools) = &request.tools {
        body["tools"] = Value::Array(tools.iter().map(tool).collect());
    }
    Ok(body)
}

/// Split the conversation into a system prompt and Messages.
fn split(messages: &[Message]) -> (Option<String>, Vec<Value>) {
    let mut system: Vec<&str> = Vec::new();
    let mut converted = Vec::new();
    for message in messages {
        match message.role {
            Role::System => system.push(&message.content),
            Role::User => converted.push(json!({ "role": "user", "content": message.content })),
            Role::Assistant => converted.push(assistant(message)),
            Role::Tool => converted.push(tool_result(message)),
        }
    }
    let system = (!system.is_empty()).then(|| system.join("\n\n"));
    (system, converted)
}

fn assistant(message: &Message) -> Value {
    let mut blocks = Vec::new();
    if !message.content.is_empty() {
        blocks.push(json!({ "type": "text", "text": message.content }));
    }
    for call in message.tool_calls.iter().flatten() {
        blocks.push(json!({
            "type": "tool_use",
            "id": call.id,
            "name": call.function.name,
            "input": arguments(&call.function.arguments),
        }));
    }
    json!({ "role": "assistant", "content": blocks })
}

fn tool_result(message: &Message) -> Value {
    json!({
        "role": "user",
        "content": [{
            "type": "tool_result",
            "tool_use_id": message.tool_call_id,
            "content": message.content,
        }],
    })
}

/// Tool arguments travel as an object here, not as a JSON string.
fn arguments(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

/// One tool in the Messages shape; an already-translated tool passes through.
fn tool(tool: &Value) -> Value {
    if tool.get("input_schema").is_some() {
        return tool.clone();
    }
    let function = tool.get("function").unwrap_or(tool);
    json!({
        "name": function.get("name").cloned().unwrap_or(Value::Null),
        "description": function.get("description").cloned().unwrap_or(Value::Null),
        "input_schema": function
            .get("parameters")
            .cloned()
            .unwrap_or(json!({ "type": "object" })),
    })
}
