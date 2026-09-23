// --- provider::wire::anthropic::parse ---
// Reading a Messages answer: text blocks become the answer, tool blocks become
// the same `ToolCall` shape the chat-completions dialect produces, and the
// token counts are renamed.

use serde::Deserialize;
use serde_json::Value;

use crate::provider::Usage;
use crate::provider::error::Error;
use crate::provider::types::{ChatResponse, Choice, FunctionCall, Message, ToolCall};

/// Parse a Messages answer.
pub(crate) fn response(provider: &str, body: &str) -> Result<ChatResponse, Error> {
    let reply: Reply = serde_json::from_str(body).map_err(|source| Error::Decode {
        provider: provider.to_owned(),
        source,
    })?;
    if reply.content.is_empty() {
        return Err(Error::Shape {
            provider: provider.to_owned(),
            message: "the answer carried no content".to_owned(),
        });
    }
    let text: String = reply
        .content
        .iter()
        .filter_map(Block::text)
        .collect::<Vec<_>>()
        .concat();
    let calls: Vec<ToolCall> = reply.content.iter().filter_map(Block::call).collect();
    let usage = reply.usage.map(|usage| Usage {
        prompt_tokens: usage.input_tokens,
        completion_tokens: usage.output_tokens,
        total_tokens: usage.input_tokens.saturating_add(usage.output_tokens),
    });
    let mut message = Message::assistant(&text);
    message.tool_calls = (!calls.is_empty()).then_some(calls);
    Ok(ChatResponse {
        choices: vec![Choice {
            message,
            finish_reason: reply.stop_reason,
        }],
        usage,
    })
}

#[derive(Debug, Deserialize)]
struct Reply {
    #[serde(default)]
    content: Vec<Block>,
    #[serde(default)]
    usage: Option<RawUsage>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    /// Anything else the dialect grows, e.g. `thinking`.
    #[serde(other)]
    Other,
}

impl Block {
    fn text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    fn call(&self) -> Option<ToolCall> {
        match self {
            Self::ToolUse { id, name, input } => Some(ToolCall {
                id: id.clone(),
                kind: "function".to_owned(),
                function: FunctionCall {
                    name: name.clone(),
                    arguments: input.to_string(),
                },
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}
