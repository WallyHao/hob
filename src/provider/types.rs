// --- provider::types ---
// The shapes exchanged with an OpenAI-compatible backend, kept apart from the
// client so the request contract can be read on its own.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::usage::Usage;

/// Who a message is from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Instructions that frame the conversation.
    System,
    /// What the user asked.
    User,
    /// What the model answered, used to replay history.
    Assistant,
    /// The result of a tool call.
    Tool,
}

impl Role {
    /// The word this role travels under.
    pub fn name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// One tool call the model asked for, as it travels on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Provider-assigned id, echoed back with the result.
    pub id: String,
    /// Always `function` in this dialect.
    #[serde(rename = "type")]
    pub kind: String,
    /// What to call, and with what.
    pub function: FunctionCall,
}

/// The function a tool call names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name.
    pub name: String,
    /// Arguments as a JSON string, exactly as the model produced them.
    pub arguments: String,
}

/// One chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Who wrote it.
    pub role: Role,
    /// Message body.
    pub content: String,
    /// Tool calls the assistant asked for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// The call a tool message answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// The model's reasoning, when it reports one. Never sent back.
    #[serde(default, skip_serializing, alias = "reasoning")]
    pub reasoning_content: Option<String>,
}

impl Message {
    /// A system message.
    pub fn system(content: &str) -> Self {
        Self::text(Role::System, content)
    }

    /// A user message.
    pub fn user(content: &str) -> Self {
        Self::text(Role::User, content)
    }

    /// An assistant message.
    pub fn assistant(content: &str) -> Self {
        Self::text(Role::Assistant, content)
    }

    fn text(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_owned(),
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }
}

/// One chat completions request.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// Model id, e.g. `deepseek-chat`.
    pub model: String,
    /// Conversation, first message first.
    pub messages: Vec<Message>,
    /// Always sent, so the wire shape does not change when streaming lands.
    pub stream: bool,
    /// Optional cap on generated tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional reasoning effort, forwarded as `reasoning_effort`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Tools the model may ask for, passed through as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
}

impl ChatRequest {
    /// A non-streaming request for one model.
    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self {
            model: model.to_owned(),
            messages,
            stream: false,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            tools: None,
        }
    }
}

/// A chat completions response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// One entry per sampled completion.
    pub choices: Vec<Choice>,
    /// Token accounting, when the provider reports it.
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl ChatResponse {
    /// The text of the first choice, the only one hob asks for.
    pub fn first_text(&self) -> Option<&str> {
        self.choices
            .first()
            .map(|choice| choice.message.content.as_str())
    }
}
/// One completion.
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    /// The assistant message.
    pub message: Message,
}
