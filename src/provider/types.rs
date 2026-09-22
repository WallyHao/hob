// --- provider::types ---
// The shapes exchanged with an OpenAI-compatible backend, kept apart from the
// client so the request contract can be read on its own.

use serde::{Deserialize, Serialize};

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
}

/// One chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Who wrote it.
    pub role: Role,
    /// Message body.
    pub content: String,
}

impl Message {
    /// A system message.
    pub fn system(content: &str) -> Self {
        Self {
            role: Role::System,
            content: content.to_owned(),
        }
    }

    /// A user message.
    pub fn user(content: &str) -> Self {
        Self {
            role: Role::User,
            content: content.to_owned(),
        }
    }

    /// An assistant message.
    pub fn assistant(content: &str) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_owned(),
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
}

impl ChatRequest {
    /// A non-streaming request for one model.
    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self {
            model: model.to_owned(),
            messages,
            stream: false,
            max_tokens: None,
        }
    }
}

/// A chat completions response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// One entry per sampled completion.
    pub choices: Vec<Choice>,
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
