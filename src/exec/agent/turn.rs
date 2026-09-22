// --- exec::agent::turn ---
// One round trip: send, and when the answer must match a schema, validate and
// ask again. Transport failures get the same attempt budget.

use std::time::Duration;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::agent::Settings;
use crate::provider::{Message, ProviderSpec, Usage};

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

/// One answer, retrying transport failures and schema mistakes.
pub(crate) fn round_trip(
    spec: &ProviderSpec,
    model: &str,
    messages_in: &[Message],
    settings: &Settings,
) -> Result<Reply, Failure> {
    let allowed = settings.max_attempts.unwrap_or(3).max(1);
    let mut conversation = messages_in.to_vec();
    if let Some(schema) = &settings.schema {
        conversation.push(Message::system(&schema::instruction(schema)));
    }
    let mut attempts = 0;
    loop {
        attempts += 1;
        let request = wire::request(model, &conversation, settings);
        let response = match provider::call(spec, &request) {
            Ok(response) => response,
            Err(_) if attempts < allowed => {
                pause(attempts);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        let message = response
            .choices
            .first()
            .map(|choice| choice.message.clone())
            .ok_or_else(|| Failure::new("the provider returned no choices"))?;
        let meta = wire::meta(spec, model, attempts, &response);
        let usage = response.usage.unwrap_or_default();
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
            Err(error) if attempts < allowed => {
                conversation.push(message);
                conversation.push(Message::user(&schema::repair(&error, schema)));
            }
            Err(error) => {
                return Err(Failure::new(format!(
                    "the model did not answer with valid JSON: {error}"
                )));
            }
        }
    }
}

/// A short pause before a retry, so a dead connection is not hammered.
fn pause(attempts: u32) {
    std::thread::sleep(Duration::from_millis(100 * u64::from(attempts)));
}
