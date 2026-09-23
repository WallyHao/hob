// --- exec::agent::chat ---
// A conversation: the messages so far, the settings it opened with and the
// usage it has accumulated.

use serde_json::{Value, json};

use crate::effect::Failure;
use crate::effect::ops::agent::Settings;
use crate::exec::budget::Budget;
use crate::provider::{Client, Message, ProviderSpec, Usage};

use super::{budget, messages, turn};

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
        client: &Client,
        prompt: Option<&str>,
        new: Option<&Vec<Value>>,
        settings: &Settings,
        budget: &Budget,
    ) -> Result<(Value, Value), Failure> {
        let new = messages::opening(new, prompt, None)?;
        let settings = settings.over(&self.settings);
        let system = self.system.as_deref().map(Message::system);
        let fixed = system.as_ref().map_or(0, budget::tokens) + budget::estimate(&new);
        let limit = settings.max_prompt_tokens.unwrap_or(budget::DEFAULT);
        let dropped = budget::dropped(&self.messages, fixed, limit);
        let mut outgoing: Vec<Message> = system.into_iter().collect();
        if dropped > 0 {
            outgoing.push(Message::system(&budget::marker(dropped, limit)));
        }
        outgoing.extend(self.messages[dropped..].iter().cloned());
        outgoing.extend(new.iter().cloned());
        let reply = turn::round_trip(
            client,
            &self.spec,
            &self.model,
            &outgoing,
            &settings,
            budget,
        )?;
        self.messages.extend(new);
        self.messages.push(reply.message.clone());
        self.usage.add(&reply.usage);
        Ok((reply.answer, with_dropped(reply.meta, dropped)))
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

    pub(crate) fn schema(&self) -> Option<&Value> {
        self.settings.schema.as_ref()
    }

    /// The provider this conversation is bound to.
    pub(crate) fn spec(&self) -> &ProviderSpec {
        &self.spec
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

/// Note how many messages a send left out, when any went.
fn with_dropped(mut meta: Value, dropped: usize) -> Value {
    if dropped > 0
        && let Some(fields) = meta.as_object_mut()
    {
        fields.insert("trimmed".to_owned(), json!(dropped));
    }
    meta
}
