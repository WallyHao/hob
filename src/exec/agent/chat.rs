// --- exec::agent::chat ---
// A conversation: the messages so far, the settings it opened with and the
// usage it has accumulated.

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::agent::Settings;
use crate::provider::{Message, ProviderSpec, Usage};

use super::{messages, turn};

/// One conversation.
#[derive(Debug)]
pub(crate) struct Chat {
    spec: ProviderSpec,
    model: String,
    system: Option<String>,
    messages: Vec<Message>,
    settings: Settings,
    usage: Usage,
}

impl Chat {
    /// Open a conversation.
    pub(crate) fn open(
        spec: ProviderSpec,
        model: String,
        system: Option<String>,
        settings: Settings,
    ) -> Self {
        Self {
            spec,
            model,
            system,
            messages: Vec::new(),
            settings,
            usage: Usage::default(),
        }
    }

    /// One turn: send the new messages and commit the exchange.
    pub(crate) fn send(
        &mut self,
        prompt: Option<&str>,
        new: Option<&Vec<Value>>,
        settings: &Settings,
    ) -> Result<(Value, Value), Failure> {
        let new = messages::opening(new, prompt, None)?;
        let mut outgoing = self
            .system
            .clone()
            .map_or_else(Vec::new, |system| vec![Message::system(&system)]);
        outgoing.extend(self.messages.iter().cloned());
        outgoing.extend(new.iter().cloned());
        let reply = turn::round_trip(
            &self.spec,
            &self.model,
            &outgoing,
            &settings.over(&self.settings),
        )?;
        self.messages.extend(new);
        self.messages.push(reply.message.clone());
        self.usage.add(&reply.usage);
        Ok((reply.answer, reply.meta))
    }

    /// Inject a message without calling the model.
    pub(crate) fn push(&mut self, message: &Value) -> Result<(), Failure> {
        let message = match message {
            Value::String(text) => Message::user(text),
            other => messages::message_of(other)?,
        };
        self.messages.push(message);
        Ok(())
    }

    /// The committed turns, oldest first.
    pub(crate) fn turns(&self) -> Value {
        Value::Array(self.messages.iter().map(messages::turn).collect())
    }

    /// The provider id and model, for a preview of a refused turn.
    pub(crate) fn label(&self) -> (&str, &str) {
        (&self.spec.id, &self.model)
    }

    /// Token accounting so far.
    pub(crate) fn usage(&self) -> Usage {
        self.usage
    }

    /// Forget the turns; the system prompt and the accounting stay.
    pub(crate) fn reset(&mut self) {
        self.messages.clear();
    }
}
