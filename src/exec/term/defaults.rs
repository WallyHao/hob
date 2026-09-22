// --- exec::term::defaults ---
// The value a question takes when nobody answers.
//
// `--yes` returns it and a preview resumes with it, so both paths agree on what
// "the default" means: the named option's value for a list, an empty string
// when the question named no default at all.

use serde_json::Value;

use crate::effect::ops::term::{Choice, Choose, Select};

/// The value of the default label, or an empty string when there is none.
pub(crate) fn select(request: &Select) -> Value {
    request
        .default
        .as_deref()
        .and_then(|label| {
            request
                .options
                .iter()
                .find(|choice| choice.label() == label)
        })
        .map_or_else(|| Value::String(String::new()), Choice::value)
}

/// The values of the default labels, in the order they are listed.
pub(crate) fn choose(request: &Choose) -> Value {
    Value::Array(
        request
            .options
            .iter()
            .filter(|choice| request.defaults.iter().any(|label| label == choice.label()))
            .map(Choice::value)
            .collect(),
    )
}
