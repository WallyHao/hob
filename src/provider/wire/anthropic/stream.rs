// --- provider::wire::anthropic::stream ---
// Messages events: `message_start` carries the input tokens, each
// `content_block_delta` a piece of text, and `message_delta` the stop reason
// with the output tokens. The assembled answer is the shape a non-streaming
// call returns.

use serde::Deserialize;

use crate::provider::Usage;
use crate::provider::error::Error;
use crate::provider::types::{ChatResponse, Choice, Message};

use super::super::stream::Decode;

/// One streaming answer being assembled.
pub(crate) struct Stream {
    provider: String,
    text: String,
    input_tokens: u64,
    output_tokens: u64,
    stop: Option<String>,
}

impl Stream {
    /// A reader for one provider.
    pub(crate) fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            text: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            stop: None,
        }
    }
}

impl Decode for Stream {
    fn feed(&mut self, data: &str) -> Result<Option<String>, Error> {
        let event: Event = serde_json::from_str(data).map_err(|source| Error::Decode {
            provider: self.provider.clone(),
            source,
        })?;
        match event.kind.as_str() {
            "message_start" => {
                if let Some(input) = event.message.and_then(|start| start.usage) {
                    self.input_tokens = input.input_tokens;
                }
            }
            "message_delta" => {
                if let Some(stop) = event.delta.and_then(|delta| delta.stop_reason) {
                    self.stop = Some(stop);
                }
                if let Some(output) = event.usage {
                    self.output_tokens = output.output_tokens;
                }
            }
            "content_block_delta" => {
                if let Some(text) = event
                    .delta
                    .and_then(|delta| delta.text)
                    .filter(|text| !text.is_empty())
                {
                    self.text.push_str(&text);
                    return Ok(Some(text));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn finish(&mut self, _provider: &str) -> Result<ChatResponse, Error> {
        Ok(ChatResponse {
            choices: vec![Choice {
                message: Message::assistant(&self.text),
                finish_reason: self.stop.take(),
            }],
            usage: Some(Usage {
                prompt_tokens: self.input_tokens,
                completion_tokens: self.output_tokens,
                total_tokens: self.input_tokens.saturating_add(self.output_tokens),
            }),
        })
    }
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    delta: Option<Delta>,
    #[serde(default)]
    message: Option<Start>,
    #[serde(default)]
    usage: Option<Output>,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct Start {
    #[serde(default)]
    usage: Option<Input>,
}

#[derive(Deserialize)]
struct Input {
    #[serde(default)]
    input_tokens: u64,
}

#[derive(Deserialize)]
struct Output {
    #[serde(default)]
    output_tokens: u64,
}
