// --- exec::agent ---
// Model calls. `ask` is one round trip; `open` returns a conversation whose
// turns live here, so a flow cannot rewrite history by accident and the
// engine can keep the accounting straight.

mod chat;
mod messages;
mod provider;
pub(crate) mod schema;
mod turn;
pub(crate) mod wire;

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::agent::{Ask, Handle, List, Open, Push, Send};
use crate::exec::State;

pub(crate) use chat::Chat;

/// One round trip.
pub(crate) fn ask(request: &Ask) -> Result<Value, Failure> {
    let spec = provider::spec(request.provider.as_ref())?;
    let model = provider::model(request.model.as_deref())?;
    let messages = messages::opening(
        request.messages.as_ref(),
        request.prompt.as_deref(),
        request.system.as_deref(),
    )?;
    let reply = turn::round_trip(&spec, &model, &messages, &request.settings)?;
    Ok(wire::result(&reply.answer, &reply.meta))
}

/// Open a conversation and return its handle.
pub(crate) fn open(state: &mut State, request: &Open) -> Result<Value, Failure> {
    let spec = provider::spec(request.provider.as_ref())?;
    let model = provider::model(request.model.as_deref())?;
    let id = state.id();
    state.chats.insert(
        id,
        Chat::open(
            spec,
            model,
            request.system.clone(),
            request.settings.clone(),
        ),
    );
    Ok(Value::from(id))
}

/// The models a provider lists.
pub(crate) fn list(request: &List) -> Result<Value, Failure> {
    let spec = provider::spec(request.provider.as_ref())?;
    let models = provider::models(&spec)?;
    Ok(Value::Array(
        models.into_iter().map(Value::String).collect(),
    ))
}

/// One turn in a conversation.
pub(crate) fn send(state: &mut State, request: &Send) -> Result<Value, Failure> {
    let chat = chat(state, request.session)?;
    let (answer, meta) = chat.send(
        request.prompt.as_deref(),
        request.messages.as_ref(),
        &request.settings,
    )?;
    Ok(wire::result(&answer, &meta))
}

/// Inject a message without calling the model.
pub(crate) fn push(state: &mut State, request: &Push) -> Result<Value, Failure> {
    chat(state, request.session)?.push(&request.message)?;
    Ok(Value::Null)
}

/// The committed turns.
pub(crate) fn turns(state: &State, request: &Handle) -> Result<Value, Failure> {
    Ok(chat_ref(state, request.session)?.turns())
}

/// Token accounting.
pub(crate) fn usage(state: &State, request: &Handle) -> Result<Value, Failure> {
    Ok(wire::usage(chat_ref(state, request.session)?.usage()))
}

/// Forget the turns.
pub(crate) fn reset(state: &mut State, request: &Handle) -> Result<Value, Failure> {
    chat(state, request.session)?.reset();
    Ok(Value::Null)
}

/// Drop the conversation.
pub(crate) fn close(state: &mut State, request: &Handle) -> Result<Value, Failure> {
    state
        .chats
        .remove(&request.session)
        .ok_or_else(|| unknown(request.session))?;
    Ok(Value::Null)
}

fn chat_ref(state: &State, id: u64) -> Result<&Chat, Failure> {
    state.chats.get(&id).ok_or_else(|| unknown(id))
}

fn chat(state: &mut State, id: u64) -> Result<&mut Chat, Failure> {
    state.chats.get_mut(&id).ok_or_else(|| unknown(id))
}

fn unknown(id: u64) -> Failure {
    Failure::new(format!("no conversation with handle {id}"))
}
