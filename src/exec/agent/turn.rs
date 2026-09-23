// --- exec::agent::turn ---
// One round trip: send, and when the answer must match a schema, validate and
// ask again. Transport failures get the same attempt budget.

use std::io::{self, Write as _};
use std::time::Duration;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::agent::Settings;
use crate::exec::budget::Budget;
use crate::provider::{Client, Message, ProviderSpec, Usage};

use super::{provider, schema, wire};

/// What one round trip produced.
pub(crate) struct Reply {
    /// The answer the flow receives: text, or decoded JSON when a schema was
    /// given.
    pub(crate) answer: Value,
    /// The assistant message to commit.
    pub(crate) message: Message,
    /// The meta table.
    pub(crate) meta: Value,
    /// Tokens the provider reported.
    pub(crate) usage: Usage,
}

/// One answer, retrying failures that could clear and schema mistakes.
pub(crate) fn round_trip(
    client: &Client,
    spec: &ProviderSpec,
    model: &str,
    messages_in: &[Message],
    settings: &Settings,
    budget: &Budget,
) -> Result<Reply, Failure> {
    let allowed = settings.max_attempts.unwrap_or(3).max(1);
    let streaming = settings.stream.unwrap_or(false);
    if streaming && settings.tools.is_some() {
        return Err(Failure::new(
            "`stream` cannot be combined with `tools` yet: a streamed tool call cannot be assembled",
        ));
    }
    let mut conversation = messages_in.to_vec();
    if let Some(schema) = &settings.schema {
        conversation.push(Message::system(&schema::instruction(schema)));
    }
    let mut attempts = 0;
    let mut usage = Usage::default();
    loop {
        attempts += 1;
        budget.call()?;
        let request = wire::request(model, &conversation, settings);
        let mut emitted = false;
        let result = provider::call(client, &request, &mut |delta| {
            emitted = true;
            show(delta);
        });
        let response = match result {
            Ok(response) => response,
            Err(error) if error.retryable && attempts < allowed && !emitted => {
                pause(attempts);
                continue;
            }
            Err(error) => return Err(error.failure),
        };
        let message = response
            .choices
            .first()
            .map(|choice| choice.message.clone())
            .ok_or_else(|| Failure::new("the provider returned no choices"))?;
        let attempt = response.usage.unwrap_or_default();
        usage.add(&attempt);
        budget.spend(attempt);
        let meta = wire::meta(spec, model, attempts, &response, usage);
        let Some(schema) = &settings.schema else {
            return Ok(Reply {
                answer: Value::String(message.content.clone()),
                message,
                meta,
                usage,
            });
        };
        match schema::extract(&message.content)
            .and_then(|value| schema::validate(schema, &value).map(|()| value))
        {
            Ok(value) => {
                return Ok(Reply {
                    answer: value,
                    message,
                    meta,
                    usage,
                });
            }
            Err(error) if attempts < allowed && !emitted => {
                conversation.push(message);
                conversation.push(Message::user(&schema::repair(&error, schema)));
            }
            Err(error) => {
                let cut = if response.truncated() {
                    "; the answer was cut off by the output cap"
                } else {
                    ""
                };
                return Err(Failure::new(format!(
                    "the model did not answer with valid JSON: {error}{cut}"
                )));
            }
        }
    }
}

/// Show a delta while the answer is still arriving. Progress goes to stderr,
/// so stdout stays the flow's own output and the answer is still returned.
fn show(delta: &str) {
    let _ = io::stderr().lock().write_all(delta.as_bytes());
}

/// A pause before a retry: 100ms, then 200, 400..., so a dead connection is
/// not hammered while a brief failure gets time to clear.
fn pause(attempts: u32) {
    let backoff = 100u64 << attempts.saturating_sub(1).min(5);
    std::thread::sleep(Duration::from_millis(backoff));
}
