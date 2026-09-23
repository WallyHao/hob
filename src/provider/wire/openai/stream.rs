// --- provider::wire::openai::stream ---
// Chat-completions deltas: `choices[0].delta` carries the text, tool calls
// arrive in fragments keyed by index, and `usage` arrives last when the request
// asked for it. The assembled answer is the shape a non-streaming call returns.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::provider::Usage;
use crate::provider::error::Error;
use crate::provider::types::{ChatResponse, Choice, FunctionCall, Message, ToolCall};

use super::super::stream::Decode;

/// One streaming answer being assembled.
pub(crate) struct Stream {
    provider: String,
    text: String,
    calls: BTreeMap<u64, Partial>,
    finish: Option<String>,
    usage: Option<Usage>,
}

impl Stream {
    /// A reader for one provider.
    pub(crate) fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            text: String::new(),
            calls: BTreeMap::new(),
            finish: None,
            usage: None,
        }
    }
}

impl Decode for Stream {
    fn feed(&mut self, data: &str) -> Result<Option<String>, Error> {
        let chunk: Value = serde_json::from_str(data).map_err(|source| Error::Decode {
            provider: self.provider.clone(),
            source,
        })?;
        if chunk.get("error").is_some() {
            return Err(Error::Shape {
                provider: self.provider.clone(),
                message: "provider reported a streaming error".to_owned(),
            });
        }
        if let Some(usage) = chunk.get("usage").filter(|usage| !usage.is_null()) {
            self.usage = serde_json::from_value(usage.clone()).ok();
        }
        let Some(choice) = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return Ok(None);
        };
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.finish = Some(reason.to_owned());
        }
        for call in choice
            .pointer("/delta/tool_calls")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
            let partial = self.calls.entry(index).or_default();
            if let Some(id) = call.get("id").and_then(Value::as_str) {
                id.clone_into(&mut partial.id);
            }
            if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                partial.name.push_str(name);
            }
            if let Some(arguments) = call.pointer("/function/arguments").and_then(Value::as_str) {
                partial.arguments.push_str(arguments);
            }
        }
        let Some(delta) = choice
            .pointer("/delta/content")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
        else {
            return Ok(None);
        };
        self.text.push_str(delta);
        Ok(Some(delta.to_owned()))
    }

    fn finish(&mut self, _provider: &str) -> Result<ChatResponse, Error> {
        if self.finish.is_none() {
            return Err(Error::Shape {
                provider: self.provider.clone(),
                message: "stream has no finish reason".to_owned(),
            });
        }
        let mut message = Message::assistant(&self.text);
        if !self.calls.is_empty() {
            message.tool_calls = Some(self.calls.values().map(flatten).collect());
        }
        Ok(ChatResponse {
            choices: vec![Choice {
                message,
                finish_reason: self.finish.take(),
            }],
            usage: self.usage.take(),
        })
    }

    fn complete(&self) -> bool {
        false
    }
}

/// One tool call, once its fragments have arrived.
fn flatten(partial: &Partial) -> ToolCall {
    ToolCall {
        id: partial.id.clone(),
        kind: "function".to_owned(),
        function: FunctionCall {
            name: partial.name.clone(),
            arguments: partial.arguments.clone(),
        },
    }
}

#[derive(Default)]
struct Partial {
    id: String,
    name: String,
    arguments: String,
}
